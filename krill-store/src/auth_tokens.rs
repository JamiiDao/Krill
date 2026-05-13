use bitcode::{Decode, Encode};
use krill_common::{AuthTokenDetails, Holder, KrillError, KrillResult, ServerConfigurationState};

use crate::KrillStorage;

pub type AuthTokenType = [u8; AuthTokenDetails::AUTH_TOKEN_LEN];

impl KrillStorage {
    pub(crate) const KEYSPACE_AUTH_TOKENS: &str = "AuthTokens";
    const KEY_SUPERUSER_AUTH_TOKENS: &str = "SuperuserAuthTokens";

    pub async fn set_auth_token(
        &self,
        token: [u8; AuthTokenDetails::BYTE_32_LEN],
        details: AuthTokenDetails,
    ) -> KrillResult<[u8; AuthTokenDetails::AUTH_TOKEN_LEN]> {
        let keyspace = self.auth_tokens_namespace();

        let auth_token_key = details.store_key(token);

        self.set(keyspace, auth_token_key, details)
            .await
            .map(|_| auth_token_key)
    }

    pub async fn set_auth_token_with_store_key(
        &self,
        auth_token_key: [u8; AuthTokenDetails::AUTH_TOKEN_LEN],
        details: AuthTokenDetails,
    ) -> KrillResult<[u8; AuthTokenDetails::AUTH_TOKEN_LEN]> {
        let keyspace = self.auth_tokens_namespace();

        self.set(keyspace, auth_token_key, details)
            .await
            .map(|_| auth_token_key)
    }

    pub async fn remove_auth_token(&self, token: AuthTokenType) -> KrillResult<()> {
        let keyspace = self.auth_tokens_namespace();
        self.remove(keyspace, token).await
    }

    pub async fn remove_superuser_auth_token(&self) -> KrillResult<()> {
        let keyspace = self.auth_tokens_namespace();

        self.remove(keyspace, Self::KEY_SUPERUSER_AUTH_TOKENS).await
    }

    /// Removes auth token if expired
    pub async fn get_auth_token(&self, token: AuthTokenType) -> KrillResult<AuthTokenDetails> {
        self.get_auth_token_checked(token)
            .await?
            .ok_or(KrillError::InvalidAuthToken)
    }

    /// Removes auth token if expired
    pub async fn get_auth_token_checked(
        &self,
        token: AuthTokenType,
    ) -> KrillResult<Option<AuthTokenDetails>> {
        let keyspace = self.auth_tokens_namespace();

        let auth_token = self
            .get(keyspace, token)
            .await?
            .map(|token_bytes| {
                bitcode::decode::<AuthTokenDetails>(&token_bytes).or(Err(KrillError::Store(
                    "Auth Token bytes corrupted".to_string(),
                )))
            })
            .transpose()?;

        if let Some(auth_token_exists) = auth_token.as_ref() {
            if auth_token_exists.is_expired() {
                self.remove_auth_token(token).await?;
            }

            return Ok(auth_token);
        }

        Ok(None)
    }

    pub async fn set_superuser_token(&self, holder: Holder) -> KrillResult<SuperuserAuthToken> {
        let app_state_keyspace = self.app_state_keyspace();
        let auth_tokens_keyspace = self.auth_tokens_namespace();

        let auth_token_details = AuthTokenDetails::new(holder);
        let auth_token: AuthTokenType =
            auth_token_details.store_key(AuthTokenDetails::generate_token());
        let token_outcome = SuperuserAuthToken {
            details: auth_token_details,
            token: auth_token,
        };

        self.set_many_with_keyspaces_and_encoded(vec![
            (
                app_state_keyspace,
                Self::KEYSPACE_APP_STATE,
                bitcode::encode(&ServerConfigurationState::Initialized),
            ),
            (
                auth_tokens_keyspace,
                Self::KEY_SUPERUSER_AUTH_TOKENS,
                bitcode::encode(&token_outcome),
            ),
        ])
        .await?;

        Ok(token_outcome)
    }

    /// Removes auth token if expired
    pub async fn get_superuser_auth_token(&self) -> KrillResult<Option<SuperuserAuthToken>> {
        let auth_tokens_keyspace = self.auth_tokens_namespace();

        let auth_token = self
            .get(auth_tokens_keyspace, Self::KEY_SUPERUSER_AUTH_TOKENS)
            .await?
            .map(|token_bytes| {
                bitcode::decode::<SuperuserAuthToken>(&token_bytes).or(Err(KrillError::Store(
                    "Auth Token bytes corrupted".to_string(),
                )))
            })
            .transpose()?;

        if let Some(auth_token_exists) = auth_token.as_ref() {
            let checked = if auth_token_exists.details.is_expired() {
                self.remove_superuser_auth_token().await?;

                None
            } else {
                auth_token
            };

            return Ok(checked);
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SuperuserAuthToken {
    pub details: AuthTokenDetails,
    pub token: AuthTokenType,
}
