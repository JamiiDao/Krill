use std::collections::{BTreeMap, HashMap};

use async_dup::Arc;
use async_lock::RwLock;
use frost_core::Ciphersuite;
use serde::{Deserialize, Serialize};

use crate::{
    FrostDistributedSigningStorage, FrostDkgError, FrostKeyStore, FrostSigningData, MessageData,
    ParticipantMessageData,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FrostSigningMemStorage<C: Ciphersuite> {
    pub keys_data: FrostSigningData<C>,
    pub coordinator_messages: BTreeMap<[u8; 32], MessageData<C>>, // Bytes for MessageData
    pub participant_messages: HashMap<[u8; 32], ParticipantMessageData<C>>, // Bytes for participant MessageData
}

impl<C: Ciphersuite> FrostSigningMemStorage<C> {
    pub fn init(keys_data: FrostSigningData<C>) -> Self {
        Self {
            keys_data,
            coordinator_messages: BTreeMap::default(),
            participant_messages: HashMap::default(),
        }
    }
}

impl<C: Ciphersuite> FrostKeyStore<C> for FrostSigningMemStorage<C> {
    fn keys_data(&self) -> &FrostSigningData<C> {
        &self.keys_data
    }

    fn get_message(&self, message_hash: &[u8; 32]) -> Option<&MessageData<C>> {
        self.coordinator_messages.get(message_hash)
    }

    fn get_mut_message(&mut self, message_hash: &[u8; 32]) -> Option<&mut MessageData<C>> {
        self.coordinator_messages.get_mut(message_hash)
    }

    fn coordinator_messages(&self) -> BTreeMap<[u8; 32], MessageData<C>> {
        self.coordinator_messages.clone()
    }

    fn participant_messages(&self) -> HashMap<[u8; 32], ParticipantMessageData<C>> {
        self.participant_messages.clone()
    }

    fn set_message(&mut self, message_hash: &[u8; 32], message_data: MessageData<C>) -> bool {
        use std::collections::btree_map::Entry;

        match self.coordinator_messages.entry(*message_hash) {
            Entry::Vacant(e) => {
                e.insert(message_data);

                true
            }
            Entry::Occupied(_) => false,
        }
    }

    fn remove_message(&mut self, message_hash: &[u8; 32]) -> Option<MessageData<C>> {
        self.coordinator_messages.remove(message_hash)
    }

    fn set_participant_message(
        &mut self,
        message_hash: &crate::Message32ByteHash,
        message_data: ParticipantMessageData<C>,
    ) -> bool {
        use std::collections::hash_map::Entry;

        match self.participant_messages.entry(*message_hash) {
            Entry::Vacant(e) => {
                e.insert(message_data);

                true
            }
            Entry::Occupied(_) => false,
        }
    }

    fn get_participant_message(
        &self,
        message_hash: &crate::Message32ByteHash,
    ) -> Option<&ParticipantMessageData<C>> {
        self.participant_messages.get(message_hash)
    }

    fn get_participant_message_mut(
        &mut self,
        message_hash: &crate::Message32ByteHash,
    ) -> Option<&mut ParticipantMessageData<C>> {
        self.participant_messages.get_mut(message_hash)
    }

    fn remove_participant_message(&mut self, message_hash: &crate::Message32ByteHash) -> bool {
        self.participant_messages.remove(message_hash).is_some()
    }
}

impl<C: Ciphersuite, E: core::error::Error + std::convert::From<FrostDkgError>>
    FrostDistributedSigningStorage<C, E> for Arc<RwLock<FrostSigningMemStorage<C>>>
{
    async fn get(
        &self,
    ) -> Result<async_lock::RwLockReadGuard<'_, impl crate::FrostKeyStore<C>>, E> {
        Ok(self.read().await)
    }

    async fn set(&self) -> Result<async_lock::RwLockWriteGuard<'_, impl FrostKeyStore<C>>, E> {
        Ok(self.write().await)
    }
}
