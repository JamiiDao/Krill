use std::sync::OnceLock;

use async_dup::Arc;
use async_lock::RwLock;
use krill_common::{AppStateMachine, KrillError, KrillResult};
use krill_store::KrillStorage;

pub static KRILL_STORAGE: OnceLock<KrillStorage> = OnceLock::new();
pub(crate) static SERVER_APP_STATE: OnceLock<Arc<RwLock<AppStateMachine>>> = OnceLock::new();

pub fn store() -> KrillResult<&'static KrillStorage> {
    KRILL_STORAGE
        .get()
        .ok_or(KrillError::GlobalStorageNotInitialized)
}

pub(crate) fn init_server_statics() -> KrillResult<()> {
    futures_lite::future::block_on(async {
        let store_init = KrillStorage::init().await?;

        KRILL_STORAGE
            .set(store_init)
            .or(Err(KrillError::GlobalStorageInitializeError))?;

        let store = store()?;

        load_app_state(store).await?;

        Ok(())
    })
}

async fn load_app_state(store: &KrillStorage) -> KrillResult<AppStateMachine> {
    let app_state = store.get_app_state().await?;

    SERVER_APP_STATE
        .set(Arc::new(RwLock::new(app_state)))
        .or(Err(KrillError::UnableToSetAppState))?;

    Ok(app_state)
}
