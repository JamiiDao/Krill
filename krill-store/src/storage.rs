use core::fmt;

use bitcode::{Decode, Encode};
use camino::Utf8PathBuf;
use fjall::{
    KeyspaceCreateOptions, PersistMode, Readable, SingleWriterTxDatabase, SingleWriterTxKeyspace,
};
use krill_common::{KrillResult, KrillUtils};

pub struct KrillStorage {
    store: SingleWriterTxDatabase,
    auth_tokens: SingleWriterTxKeyspace,
    org_info: SingleWriterTxKeyspace,
    app_state: SingleWriterTxKeyspace,
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
            let store = SingleWriterTxDatabase::builder(path).open()?;

            let auth_tokens = store.keyspace(Self::KEYSPACE_AUTH_TOKENS, || {
                KeyspaceCreateOptions::default()
            })?;

            #[allow(clippy::redundant_closure)]
            let org_info =
                store.keyspace(Self::KEYSPACE_ORG_INFO, || KeyspaceCreateOptions::default())?;

            let app_state = store.keyspace(Self::KEYSPACE_APP_STATE, || {
                KeyspaceCreateOptions::default()
            })?;

            Ok(Self {
                store,
                auth_tokens,
                org_info,
                app_state,
            })
        })
        .await
    }

    pub fn db(&self) -> fjall::SingleWriterTxDatabase {
        self.store.clone()
    }

    pub async fn set(
        &self,
        keyspace: fjall::SingleWriterTxKeyspace,
        key: impl AsRef<[u8]> + Send + 'static,
        value: impl Encode + Decode<'_>,
    ) -> KrillResult<()> {
        let db = self.db();

        let value = bitcode::encode(&value);

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            tx.insert(&keyspace, key.as_ref(), &value);

            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn set_many(
        &self,
        keyspace: fjall::SingleWriterTxKeyspace,
        kvs: Vec<(
            impl AsRef<[u8]> + Send + 'static,
            impl Encode + Decode<'_> + Send + 'static,
        )>,
    ) -> KrillResult<()> {
        let db = self.db();

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            for (key, value) in kvs {
                tx.insert(&keyspace, key.as_ref(), bitcode::encode(&value));
            }

            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn set_many_with_keyspaces_and_encoded(
        &self,
        data: Vec<(
            fjall::SingleWriterTxKeyspace,
            impl AsRef<[u8]> + Send + 'static,
            Vec<u8>,
        )>,
    ) -> KrillResult<()> {
        let db = self.db();

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            for (keyspace, key, value) in data {
                tx.insert(&keyspace, key.as_ref(), value);
            }

            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn set_many_encoded(
        &self,
        keyspace: fjall::SingleWriterTxKeyspace,
        kvs: Vec<(impl AsRef<[u8]> + Send + 'static, Vec<u8>)>,
    ) -> KrillResult<()> {
        let db = self.db();

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            for (key, value) in kvs {
                tx.insert(&keyspace, key.as_ref(), &value);
            }

            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn remove(
        &self,
        keyspace: fjall::SingleWriterTxKeyspace,
        key: impl AsRef<[u8]> + Send + 'static,
    ) -> KrillResult<()> {
        let db = self.db();

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            tx.remove(&keyspace, key.as_ref());
            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn remove_many(
        &self,
        keyspace: fjall::SingleWriterTxKeyspace,
        keys: Vec<impl AsRef<[u8]> + Send + 'static>,
    ) -> KrillResult<()> {
        let db = self.db();

        blocking::unblock(move || {
            let mut tx = db.write_tx();

            for key in keys {
                tx.remove(&keyspace, key.as_ref());
            }

            tx.commit()?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn get(
        &self,
        keyspace: fjall::SingleWriterTxKeyspace,
        key: impl AsRef<[u8]> + Send + 'static,
    ) -> KrillResult<Option<Vec<u8>>> {
        let db = self.db();

        Ok(blocking::unblock(move || {
            let tx = db.read_tx();

            tx.get(keyspace, key.as_ref())
        })
        .await?
        .map(|data| data.to_vec()))
    }

    pub fn org_info_keyspace(&self) -> SingleWriterTxKeyspace {
        self.org_info.clone()
    }

    pub fn app_state_keyspace(&self) -> SingleWriterTxKeyspace {
        self.app_state.clone()
    }

    pub fn auth_tokens_namespace(&self) -> SingleWriterTxKeyspace {
        self.auth_tokens.clone()
    }
}

impl fmt::Debug for KrillStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KrillStorage")
    }
}
