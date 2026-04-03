use krill_common::{KrillError, KrillResult, ServerConfigurationState};

use crate::KrillStorage;

impl KrillStorage {
    pub const KEYSPACE_APP_STATE: &str = "AppState";

    pub async fn set_app_state_login_init(&self) -> KrillResult<()> {
        let keyspace = self.app_state_keyspace();

        self.set_op(
            keyspace,
            Self::KEYSPACE_APP_STATE,
            ServerConfigurationState::LoginInitialization,
        )
        .await
    }

    pub async fn set_app_state_initialized(&self) -> KrillResult<()> {
        let keyspace = self.app_state_keyspace();

        self.set_op(
            keyspace,
            Self::KEYSPACE_APP_STATE,
            ServerConfigurationState::Initialized,
        )
        .await
    }

    pub async fn get_app_state(&self) -> KrillResult<ServerConfigurationState> {
        let keyspace = self.app_state_keyspace();

        self.get_app_state_inner(keyspace).await
    }

    async fn get_app_state_inner(
        &self,
        keyspace: async_dup::Arc<fjall::SingleWriterTxKeyspace>,
    ) -> KrillResult<ServerConfigurationState> {
        let value: Option<Vec<u8>> = self
            .get_op(keyspace.clone(), Self::KEYSPACE_APP_STATE)
            .await?;

        if let Some(bytes) = value {
            bitcode::decode::<ServerConfigurationState>(&bytes)
                .or(Err(KrillError::UnableToDeserializeAppStateData))
        } else {
            let app_state = ServerConfigurationState::default();

            self.set_op(keyspace, Self::KEYSPACE_APP_STATE, app_state)
                .await?;

            Ok(app_state)
        }
    }
}
