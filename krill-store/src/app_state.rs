use krill_common::{AppStateMachine, KrillError, KrillResult};

use crate::{KrillStorage, KrillStoreKeyspace};

impl KrillStorage {
    pub async fn transition_app_state(&self) -> KrillResult<AppStateMachine> {
        let keyspace = self.app_state_keyspace();
        let state = self.get_app_state_inner(keyspace.clone()).await?;

        let new_state = match state {
            AppStateMachine::SetLanguage => AppStateMachine::SetColorScheme,
            AppStateMachine::SetColorScheme => AppStateMachine::SetOrganizationInfo,
            AppStateMachine::SetOrganizationInfo => AppStateMachine::SetAdministrators,
            AppStateMachine::SetAdministrators => AppStateMachine::Configured,
            AppStateMachine::Configured => return Ok(state),
        };

        self.set_op(keyspace, KrillStoreKeyspace::AppState.as_str(), new_state)
            .await?;

        Ok(new_state)
    }

    pub async fn get_app_state(&self) -> KrillResult<AppStateMachine> {
        let keyspace = self.app_state_keyspace();

        self.get_app_state_inner(keyspace).await
    }

    async fn get_app_state_inner(
        &self,
        keyspace: async_dup::Arc<fjall::SingleWriterTxKeyspace>,
    ) -> KrillResult<AppStateMachine> {
        let value = self
            .get_op(keyspace.clone(), KrillStoreKeyspace::AppState.as_str())
            .await;

        match value {
            Ok(bytes) => {
                assert_eq!(bytes, vec![0]);
                bitcode::decode::<AppStateMachine>(&bytes)
            }
            .or(Err(KrillError::UnableToDeserializeAppStateData)),
            Err(error) => match error {
                KrillError::KeyNotFoundInStore(_) => {
                    let app_state = AppStateMachine::SetLanguage;

                    self.set_op(keyspace, KrillStoreKeyspace::AppState.as_str(), app_state)
                        .await?;

                    Ok(app_state)
                }
                _ => Err(error),
            },
        }
    }
}
