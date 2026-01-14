use std::{marker::PhantomData, sync::Arc};

use camino::Utf8Path;
use fjall::{KeyspaceCreateOptions, PersistMode, SingleWriterTxDatabase, SingleWriterWriteTx};
use frost_core::Ciphersuite;

use crate::{
    FrostDkgData, KrillError, KrillResult, StoreKeys, StoreKeyspace,
};

#[derive(Clone)]
pub struct FrostStore<C: Ciphersuite + Send + Sync>(
    Arc<fjall::SingleWriterTxDatabase>,
    PhantomData<C>,
);

impl<C> FrostStore<C>
where
    C: Ciphersuite + Send + Sync,
{
    pub async fn init(path: &Utf8Path) -> KrillResult<Self> {
        let path = path.to_path_buf();

        blocking::unblock(move || {
            let db = SingleWriterTxDatabase::builder(path).open()?;

            Ok(Self(Arc::new(db), PhantomData))
        })
        .await
    }

    pub fn store(&self) -> Arc<fjall::SingleWriterTxDatabase> {
        self.0.clone()
    }

    pub async fn keypair_keyspace(&self) -> KrillResult<Arc<fjall::SingleWriterTxKeyspace>> {
        let db = self.store();

        let keyspace = blocking::unblock(move || {
            db.keyspace(StoreKeyspace::FrostKeypair.to_str(), || {
                KeyspaceCreateOptions::default()
            })
        })
        .await?;

        Ok(Arc::new(keyspace))
    }

    pub async fn get_dkg_data(&self) -> KrillResult<Vec<u8>> {
        let keyspace = self.keypair_keyspace().await?;

        blocking::unblock(move || keyspace.get(StoreKeys::Dkg.to_str()))
            .await?
            .map(|data| data.to_vec())
            .ok_or(KrillError::FrostKeypairDataNotFound)
    }

    pub async fn get_and_deserialize_dkg_data(&self) -> KrillResult<FrostDkgData> {
        let data_bytes = self.get_dkg_data().await?;

        bitcode::decode(&data_bytes).or(Err(KrillError::UnableToDeserializeIntoFrostDkgData))
    }

    pub async fn coordinator_messages_keyspace(
        &self,
    ) -> KrillResult<Arc<fjall::SingleWriterTxKeyspace>> {
        let db = self.store();

        let keyspace = blocking::unblock(move || {
            db.keyspace(StoreKeyspace::CoordinatorMessages.to_str(), || {
                KeyspaceCreateOptions::default()
            })
        })
        .await?;

        Ok(Arc::new(keyspace))
    }

    pub async fn participant_messages_keyspace(
        &self,
    ) -> KrillResult<Arc<fjall::SingleWriterTxKeyspace>> {
        let db = self.store();

        let keyspace = blocking::unblock(move || {
            db.keyspace(StoreKeyspace::ParticipantMessages.to_str(), || {
                KeyspaceCreateOptions::default()
            })
        })
        .await?;

        Ok(Arc::new(keyspace))
    }

    pub async fn signed_messages_keyspace(
        &self,
    ) -> KrillResult<Arc<fjall::SingleWriterTxKeyspace>> {
        let db = self.store();

        let keyspace = blocking::unblock(move || {
            db.keyspace(StoreKeyspace::SignedMessages.to_str(), || {
                KeyspaceCreateOptions::default()
            })
        })
        .await?;

        Ok(Arc::new(keyspace))
    }

    pub async fn set_dkg_op(&self, key: StoreKeys, bytes: Vec<u8>) -> crate::KrillResult<()> {
        let db = self.store();
        let keyspace = self.keypair_keyspace().await?;

        blocking::unblock(move || {
            // Start a single-writer transaction
            let tx: SingleWriterWriteTx = db.write_tx();

            // Perform multiple operations atomically
            keyspace.insert(key.to_str(), bytes)?;

            // Commit the transaction (ensures atomicity & durability)
            tx.commit()?;

            // Optionally persist to disk
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
            // Start a single-writer transaction
            let tx: SingleWriterWriteTx = db.write_tx();

            // Perform multiple operations atomically
            keyspace.insert(key, bytes)?;

            // Commit the transaction (ensures atomicity & durability)
            tx.commit()?;

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
