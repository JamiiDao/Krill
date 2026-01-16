use core::fmt;
use std::sync::OnceLock;

use async_dup::Arc;
use bitcode::{Decode, Encode};
use camino::Utf8PathBuf;
use fjall::{KeyspaceCreateOptions, PersistMode, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use krill_common::{KrillError, KrillResult, KrillUtils};

pub struct KrillStorage {
    store: Arc<fjall::SingleWriterTxDatabase>,
    branding: Arc<SingleWriterTxKeyspace>,
    app_state: Arc<SingleWriterTxKeyspace>,
}

impl KrillStorage {
    pub async fn init() -> KrillResult<Self> {
        let mut path = KrillUtils::krill_dir().await?;
        path.push("KrillAppStorage");

        KrillUtils::create_recursive_dir(&path).await?;

        Self::init_db(path).await
    }

    pub async fn init_db(path: Utf8PathBuf) -> KrillResult<Self> {
        blocking::unblock(move || {
            let db = SingleWriterTxDatabase::builder(path).open()?;

            let branding = db.keyspace(KrillStoreKeyspace::Branding.as_str(), || {
                KeyspaceCreateOptions::default()
            })?;

            let app_state = db.keyspace(KrillStoreKeyspace::AppState.as_str(), || {
                KeyspaceCreateOptions::default()
            })?;

            Ok(Self {
                store: Arc::new(db),
                branding: Arc::new(branding),
                app_state: Arc::new(app_state),
            })
        })
        .await
    }

    pub fn db(&self) -> Arc<fjall::SingleWriterTxDatabase> {
        self.store.clone()
    }

    pub async fn set_op(
        &self,
        keyspace: Arc<fjall::SingleWriterTxKeyspace>,
        key: impl AsRef<str>,
        value: impl Encode + Decode<'_>,
    ) -> KrillResult<()> {
        let db = self.db();

        let key = key.as_ref().to_owned();
        let value = bitcode::encode(&value);

        blocking::unblock(move || {
            // Perform multiple operations atomically
            keyspace.insert(key, value)?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn get_op(
        &self,
        keyspace: Arc<fjall::SingleWriterTxKeyspace>,
        key: impl AsRef<str>,
    ) -> KrillResult<Vec<u8>> {
        let key_clone = key.as_ref().to_owned();

        blocking::unblock(move || keyspace.get(&key_clone))
            .await?
            .map(|data| data.to_vec())
            .ok_or(KrillError::KeyNotFoundInStore(key.as_ref().to_string()))
    }

    pub fn branding_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.branding.clone()
    }

    pub fn app_state_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.app_state.clone()
    }
}

impl fmt::Debug for KrillStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KrillStorage")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum KrillStoreKeyspace {
    Branding,
    AppState,
}

impl KrillStoreKeyspace {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Branding => "Branding",
            Self::AppState => "AppState",
        }
    }
}
