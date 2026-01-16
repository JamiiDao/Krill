use core::fmt;

use async_dup::Arc;
use bitcode::{Decode, Encode};
use camino::Utf8PathBuf;
use fjall::{KeyspaceCreateOptions, PersistMode, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use krill_common::{KrillError, KrillResult, KrillUtils};

pub struct KrillStorage {
    store: Arc<fjall::SingleWriterTxDatabase>,
    secrets: Arc<SingleWriterTxKeyspace>,
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

            #[allow(clippy::redundant_closure)]
            let secrets = db.keyspace(Self::KEYSPACE_SERVER_SECRET, || {
                KeyspaceCreateOptions::default()
            })?;

            #[allow(clippy::redundant_closure)]
            let branding =
                db.keyspace(Self::KEYSPACE_BRANDING, || KeyspaceCreateOptions::default())?;

            let app_state = db.keyspace(Self::KEYSPACE_APP_STATE, || {
                KeyspaceCreateOptions::default()
            })?;

            Ok(Self {
                store: Arc::new(db),
                secrets: Arc::new(secrets),
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
    ) -> KrillResult<Option<Vec<u8>>> {
        let key_clone = key.as_ref().to_owned();

        Ok(blocking::unblock(move || keyspace.get(&key_clone))
            .await?
            .map(|data| data.to_vec()))
    }

    pub fn secrets_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.secrets.clone()
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
