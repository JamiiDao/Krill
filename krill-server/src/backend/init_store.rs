use std::sync::OnceLock;

use dioxus::prelude::*;
use krill_common::{KrillError, KrillResult};
use krill_store::KrillStorage;

pub static SERVER_KEY: OnceLock<[u8; 32]> = OnceLock::new();
pub static KRILL_STORAGE: OnceLock<KrillStorage> = OnceLock::new();
pub static SERVER_COLOR_SCHEME: OnceLock<Vec<u8>> = OnceLock::new();

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

        crate::backend::state::load_app_state(store).await?;
        load_server_key(store).await?;
        load_color_scheme(store).await
    })
}

async fn load_server_key(store: &KrillStorage) -> KrillResult<()> {
    let secret = store.get_server_secret().await?;

    SERVER_KEY
        .set(secret)
        .or(Err(KrillError::UnableToSetServerSecret))
}

async fn load_color_scheme(store: &KrillStorage) -> KrillResult<()> {
    let scheme = store.get_branding_data_bytes().await?;

    SERVER_COLOR_SCHEME
        .set(scheme)
        .or(Err(KrillError::UnableToGetColorScheme))
}
