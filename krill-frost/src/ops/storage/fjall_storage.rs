use std::{io::ErrorKind, marker::PhantomData, path::Path, sync::Arc};

use async_fs::DirBuilder;
use camino::Utf8PathBuf;
use fjall::{KeyspaceCreateOptions, PersistMode, SingleWriterTxDatabase, SingleWriterTxKeyspace};
use frost_core::Ciphersuite;

use crate::{FrostDkgData, KrillError, KrillResult, StoreKeys, StoreKeyspace};

#[derive(Clone)]
pub struct FrostStore<C: Ciphersuite + Send + Sync> {
    store: Arc<fjall::SingleWriterTxDatabase>,
    keypair_keyspace: Arc<SingleWriterTxKeyspace>,
    coordinator_messages_keyspace: Arc<SingleWriterTxKeyspace>,
    participant_messages_keyspace: Arc<SingleWriterTxKeyspace>,
    signed_messages_keyspace: Arc<SingleWriterTxKeyspace>,
    foo: PhantomData<C>,
}

impl<C> FrostStore<C>
where
    C: Ciphersuite + Send + Sync,
{
    pub async fn new() -> KrillResult<Self> {
        let krill_dir = Self::krill_dir().await?;

        Self::init(krill_dir).await
    }

    pub async fn init_with_dir(dir_name: impl AsRef<str>) -> KrillResult<Self> {
        let mut krill_dir = Self::krill_dir().await?;
        krill_dir.push(dir_name.as_ref());

        Self::init(krill_dir).await
    }

    pub async fn init_custom_path(path: impl AsRef<Path>) -> KrillResult<Self> {
        let path = path.as_ref().to_path_buf();
        let path = Utf8PathBuf::from_path_buf(path).or(Err(KrillError::HomeDirPathNotUtf8))?;

        if let Some(error) = DirBuilder::new().recursive(true).create(&path).await.err() {
            if error.kind() != ErrorKind::AlreadyExists {
                return Err(KrillError::Io(error.kind()));
            }
        }

        Self::init_db(path).await
    }

    pub async fn krill_dir() -> KrillResult<Utf8PathBuf> {
        let mut db_dir = dirs::home_dir()
            .map(|value| Utf8PathBuf::from_path_buf(value).or(Err(KrillError::HomeDirPathNotUtf8)))
            .transpose()?
            .ok_or(KrillError::UnableToFindHomeDirectory)?;
        db_dir.push(".Krill");

        Ok(db_dir)
    }

    async fn init(path: Utf8PathBuf) -> KrillResult<Self> {
        let mut path = path;
        path.push("KrillFrostStore");

        if let Some(error) = DirBuilder::new().recursive(true).create(&path).await.err() {
            if error.kind() != ErrorKind::AlreadyExists {
                return Err(KrillError::Io(error.kind()));
            }
        }

        Self::init_db(path).await
    }

    async fn init_db(path: Utf8PathBuf) -> KrillResult<Self> {
        blocking::unblock(move || {
            let db = SingleWriterTxDatabase::builder(path).open()?;

            let keypair_keyspace = db.keyspace(StoreKeyspace::FrostKeypair.to_str(), || {
                KeyspaceCreateOptions::default()
            })?;

            let coordinator_messages_keyspace = db
                .keyspace(StoreKeyspace::CoordinatorMessages.to_str(), || {
                    KeyspaceCreateOptions::default()
                })?;

            let participant_messages_keyspace = db
                .keyspace(StoreKeyspace::ParticipantMessages.to_str(), || {
                    KeyspaceCreateOptions::default()
                })?;

            let signed_messages_keyspace = db
                .keyspace(StoreKeyspace::SignedMessages.to_str(), || {
                    KeyspaceCreateOptions::default()
                })?;

            Ok(Self {
                store: Arc::new(db),
                keypair_keyspace: Arc::new(keypair_keyspace),
                coordinator_messages_keyspace: Arc::new(coordinator_messages_keyspace),
                participant_messages_keyspace: Arc::new(participant_messages_keyspace),
                signed_messages_keyspace: Arc::new(signed_messages_keyspace),
                foo: PhantomData,
            })
        })
        .await
    }

    pub fn store(&self) -> Arc<fjall::SingleWriterTxDatabase> {
        self.store.clone()
    }

    pub fn keypair_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.keypair_keyspace.clone()
    }

    pub fn coordinator_messages_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.coordinator_messages_keyspace.clone()
    }

    pub fn participant_messages_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.participant_messages_keyspace.clone()
    }

    pub fn signed_messages_keyspace(&self) -> Arc<SingleWriterTxKeyspace> {
        self.signed_messages_keyspace.clone()
    }

    pub async fn get_dkg_data(&self) -> KrillResult<Option<Vec<u8>>> {
        let keyspace = self.keypair_keyspace();

        Ok(
            blocking::unblock(move || keyspace.get(StoreKeys::Dkg.to_str()))
                .await?
                .map(|data| data.to_vec()),
        )
    }

    pub async fn get_and_deserialize_dkg_data(&self) -> KrillResult<FrostDkgData> {
        let dkg_data = self
            .get_dkg_data()
            .await?
            .map(|value| {
                bitcode::decode::<FrostDkgData>(&value)
                    .or(Err(KrillError::UnableToDeserializeFrostDkgData))
            })
            .transpose()?
            .unwrap_or_default();

        Ok(dkg_data)
    }

    pub async fn set_dkg_op(&self, key: StoreKeys, bytes: Vec<u8>) -> KrillResult<()> {
        let db = self.store();
        let keyspace = self.keypair_keyspace();

        blocking::unblock(move || {
            keyspace.insert(key.to_str(), bytes)?;

            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn set_op(
        &self,
        keyspace: Arc<fjall::SingleWriterTxKeyspace>,
        key: [u8; 32],
        bytes: Vec<u8>,
    ) -> crate::KrillResult<()> {
        let db = self.store();

        blocking::unblock(move || {
            // Perform multiple operations atomically
            keyspace.insert(key, bytes)?;

            // Optionally persist to disk
            db.persist(PersistMode::SyncAll)?;

            Ok(())
        })
        .await
    }

    pub async fn get_op(
        &self,
        keyspace: Arc<fjall::SingleWriterTxKeyspace>,
        key: [u8; 32],
        error: KrillError,
    ) -> KrillResult<Vec<u8>> {
        blocking::unblock(move || keyspace.get(key))
            .await?
            .map(|data| data.to_vec())
            .ok_or(error)
    }
}
