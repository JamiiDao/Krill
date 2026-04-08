use fjall::PersistMode;
use krill_common::{AuthTokenDetails, Holder, KrillError, KrillResult, ServerConfigurationState};

use crate::KrillStorage;

impl KrillStorage {
    pub(crate) const KEYSPACE_AUTH_TOKENS: &str = "AuthTokens";

    pub async fn set_app_init_and_cookies(
        &self,
        auth_token: blake3::Hash,
    ) -> KrillResult<AuthTokenDetails> {
        let db = self.db();

        let app_state_keyspace = self.app_state_keyspace();

        let org_info = self.get_org_info().await?;
        let auth_tokens_keyspace = self.auth_tokens_namespace();

        let holder = Holder::new_with_tld(&org_info.support_mail)?
            .set_superuser()
            .set_user_display(&org_info.name);

        let auth_token_details = AuthTokenDetails::new(holder);
        let auth_token_details_bytes = bitcode::encode(&auth_token_details);

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            tx.insert(
                &app_state_keyspace,
                Self::KEYSPACE_APP_STATE,
                bitcode::encode(&ServerConfigurationState::Initialized),
            );

            tx.insert(
                &auth_tokens_keyspace,
                auth_token.to_string(),
                auth_token_details_bytes,
            );

            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok::<(), KrillError>(())
        })
        .await?;

        Ok(auth_token_details)
    }

    pub async fn set_auth_token(
        &self,
        token: blake3::Hash,
        details: &AuthTokenDetails,
    ) -> KrillResult<()> {
        let keyspace = self.auth_tokens_namespace();

        let db = self.db();
        let value = bitcode::encode(details);

        blocking::unblock(move || {
            // Perform multiple operations atomically
            keyspace.insert(token.to_string(), value)?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn remove_auth_token(&self, token: blake3::Hash) -> KrillResult<()> {
        let keyspace = self.auth_tokens_namespace();

        let db = self.db();

        blocking::unblock(move || {
            // Perform multiple operations atomically
            keyspace.remove(token.to_string())?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    /// Removes auth token if expired
    pub async fn get_auth_token(
        &self,
        token: blake3::Hash,
    ) -> KrillResult<Option<AuthTokenDetails>> {
        let keyspace = self.auth_tokens_namespace();

        let auth_token = self
            .get_op(keyspace, token.to_string())
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

            Ok(auth_token)
        } else {
            Ok(None)
        }
    }
}
