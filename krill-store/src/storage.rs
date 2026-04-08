use core::fmt;

use async_dup::Arc;
use bitcode::{Decode, Encode};
use camino::Utf8PathBuf;
use fjall::{KeyspaceCreateOptions, PersistMode, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use krill_common::{KrillResult, KrillUtils};

pub struct KrillStorage {
    store: Arc<fjall::SingleWriterTxDatabase>,
    auth_tokens: Arc<SingleWriterTxKeyspace>,
    org_info: Arc<SingleWriterTxKeyspace>,
    app_state: Arc<SingleWriterTxKeyspace>,
    languages: Arc<SingleWriterTxKeyspace>,
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
            let auth_tokens = db.keyspace(Self::KEYSPACE_AUTH_TOKENS, || {
                KeyspaceCreateOptions::default()
            })?;

            #[allow(clippy::redundant_closure)]
            let org_info =
                db.keyspace(Self::KEYSPACE_ORG_INFO, || KeyspaceCreateOptions::default())?;

            let app_state = db.keyspace(Self::KEYSPACE_APP_STATE, || {
                KeyspaceCreateOptions::default()
            })?;

            let languages = db.keyspace(Self::KEYSPACE_SUPPORTED_LANGUAGES, || {
                KeyspaceCreateOptions::default()
            })?;

            Ok(Self {
                store: Arc::new(db),
                auth_tokens: Arc::new(auth_tokens),
                org_info: Arc::new(org_info),
                app_state: Arc::new(app_state),
                languages: Arc::new(languages),
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

    pub async fn set_op_many(
        &self,
        keyspace: Arc<fjall::SingleWriterTxKeyspace>,
        kvs: Vec<(String, Vec<u8>)>,
    ) -> KrillResult<()> {
        let db = self.db();

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            for (key, value) in kvs {
                tx.insert(&keyspace, key, value);
            }

            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub(crate) async fn set_op_encoded(
        &self,
        keyspace: Arc<fjall::SingleWriterTxKeyspace>,
        key: impl AsRef<str>,
        value: Vec<u8>,
    ) -> KrillResult<()> {
        let db = self.db();

        let key = key.as_ref().to_owned();

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
    pub fn org_info_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.org_info.clone()
    }

    pub fn app_state_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.app_state.clone()
    }

    pub fn auth_tokens_namespace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.auth_tokens.clone()
    }

    pub fn languages_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.languages.clone()
    }

    pub async fn remove_op(
        &self,
        keyspace: Arc<fjall::SingleWriterTxKeyspace>,
        key: impl AsRef<str>,
    ) -> KrillResult<()> {
        let db = self.db();

        let key = key.as_ref().to_owned();

        blocking::unblock(move || {
            keyspace.remove(key)?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }
}

impl fmt::Debug for KrillStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KrillStorage")
    }
}
