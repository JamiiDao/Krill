use krill_common::{KrillError, KrillResult};

use crate::KrillStorage;

impl KrillStorage {
    pub const KEYSPACE_APP_STATE: &str = "AppState";

    pub async fn app_now_configured(&self) -> KrillResult<bool> {
        let keyspace = self.app_state_keyspace();
        if !self.get_app_state_inner(keyspace.clone()).await? {
            self.set_op(keyspace, Self::KEYSPACE_APP_STATE, true)
                .await?;
        }

        Ok(true)
    }

    pub async fn get_app_state(&self) -> KrillResult<bool> {
        let keyspace = self.app_state_keyspace();

        self.get_app_state_inner(keyspace).await
    }

    async fn get_app_state_inner(
        &self,
        keyspace: async_dup::Arc<fjall::SingleWriterTxKeyspace>,
    ) -> KrillResult<bool> {
        let value: Option<Vec<u8>> = self
            .get_op(keyspace.clone(), Self::KEYSPACE_APP_STATE)
            .await?;

        if let Some(bytes) = value {
            bitcode::decode::<bool>(&bytes).or(Err(KrillError::UnableToDeserializeAppStateData))
        } else {
            let app_state = false;

            self.set_op(keyspace, Self::KEYSPACE_APP_STATE, app_state)
                .await?;

            Ok(app_state)
        }
    }
}
