use frost_core::Ciphersuite;

use crate::{
    CoordinatorMessageData, CoordinatorMessages, FrostDistributedSigningOps, FrostIdentifier,
    FrostKeypairData, FrostStore, KrillError, KrillResult, Message32ByteHash,
    ParticipantMessageData, ParticipantMessages, SignedMessageData, SignedMessages, StoreKeys,
};

impl<C: Ciphersuite + Send + Sync> FrostDistributedSigningOps<C> for FrostStore<C> {
    async fn set_keypair_data(&self, frost_keypair_data: &FrostKeypairData) -> KrillResult<()> {
        let frost_keypair_bytes = bitcode::encode(frost_keypair_data);
        self.set_dkg_op(StoreKeys::KeypairData, frost_keypair_bytes)
            .await
    }

    async fn set_coordinator_message(&self, message: &CoordinatorMessageData) -> KrillResult<()> {
        let message_bytes = bitcode::encode(message);

        let keyspace = self.coordinator_messages_keyspace().await?;

        self.set_op(keyspace, message.message_hash, message_bytes)
            .await
    }

    async fn set_participant_message(&self, message: &ParticipantMessageData) -> KrillResult<()> {
        let message_bytes = bitcode::encode(message);

        let keyspace = self.participant_messages_keyspace().await?;

        self.set_op(keyspace, message.message_hash, message_bytes)
            .await
    }

    async fn set_signed_message(&self, signed_message_data: &SignedMessageData) -> KrillResult<()> {
        let message_bytes = bitcode::encode(signed_message_data);
        let keyspace = self.signed_messages_keyspace().await?;

        self.set_op(keyspace, signed_message_data.message_hash, message_bytes)
            .await
    }

    async fn get_keypair_data(&self) -> KrillResult<FrostKeypairData> {
        let keyspace = self.keypair_keyspace().await?;

        let data_bytes = blocking::unblock(move || keyspace.get(StoreKeys::KeypairData.to_str()))
            .await?
            .map(|data| data.to_vec())
            .ok_or(KrillError::FrostKeypairDataNotFound)?;

        bitcode::decode(&data_bytes).or(Err(KrillError::UnableToDeserializeFrostKeypairData))
    }

    async fn get_identifier(&self) -> KrillResult<frost_core::Identifier<C>> {
        self.get_keypair_data().await?.identifier.decode()
    }

    fn is_valid_participant(
        &self,
        participant: &frost_core::Identifier<C>,
        frost_keypair_data: &FrostKeypairData,
    ) -> bool {
        frost_keypair_data
            .participants
            .iter()
            .any(|stored_participant| stored_participant == &FrostIdentifier::encode(participant))
    }

    async fn get_coordinator_messages(&self) -> KrillResult<CoordinatorMessages> {
        let keyspace = self.coordinator_messages_keyspace().await?;

        let values = keyspace
            .as_ref()
            .as_ref()
            .iter()
            .map(|key_value| key_value.value().map(|value| value.to_vec()))
            .collect::<Result<Vec<Vec<u8>>, fjall::Error>>()?;

        let mut outcome = CoordinatorMessages::default();

        values.into_iter().try_for_each(|value| {
            let message = bitcode::decode::<CoordinatorMessageData>(&value)
                .or(Err(KrillError::UnableToDeserializeCoordinatorMessages))?;

            outcome.insert(message.message_hash, message);

            Ok::<(), KrillError>(())
        })?;

        Ok(outcome)
    }

    async fn get_coordinator_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> KrillResult<CoordinatorMessageData> {
        let keyspace = self.coordinator_messages_keyspace().await?;

        self.get_op(keyspace, *message_hash, KrillError::CoordinatorDataNotFound)
            .await
            .map(|value| {
                bitcode::decode(&value)
                    .or(Err(KrillError::UnableToDeserializeCoordinatorDataNotFound))
            })?
    }

    async fn get_signed_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> KrillResult<SignedMessageData> {
        let keyspace = self.signed_messages_keyspace().await?;

        self.get_op(
            keyspace,
            *message_hash,
            KrillError::SignedMessagesDataNotFound,
        )
        .await
        .map(|value| {
            bitcode::decode(&value).or(Err(
                KrillError::UnableToDeserializeSignedMessagesDataNotFound,
            ))
        })?
    }

    async fn get_participant_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> KrillResult<ParticipantMessageData> {
        let keyspace = self.participant_messages_keyspace().await?;

        self.get_op(
            keyspace,
            *message_hash,
            KrillError::ParticipantMessagesDataNotFound,
        )
        .await
        .map(|value| {
            bitcode::decode(&value).or(Err(KrillError::UnableToDeserializeParticipantMessageData))
        })?
    }

    async fn get_participant_messages(&self) -> KrillResult<ParticipantMessages> {
        let keyspace = self.participant_messages_keyspace().await?;

        let values = keyspace
            .as_ref()
            .as_ref()
            .iter()
            .map(|value| value.value().map(|value| value.to_vec()))
            .collect::<Result<Vec<Vec<u8>>, fjall::Error>>()?;

        let mut outcome = ParticipantMessages::default();

        values.into_iter().try_for_each(|value| {
            let message = bitcode::decode::<ParticipantMessageData>(&value)
                .or(Err(KrillError::UnableToDeserializeParticipantMessages))?;

            outcome.insert(message.message_hash, message);

            Ok::<(), KrillError>(())
        })?;

        Ok(outcome)
    }

    async fn get_signed_messages(&self) -> KrillResult<SignedMessages> {
        let keyspace = self.participant_messages_keyspace().await?;

        let values = keyspace
            .as_ref()
            .as_ref()
            .iter()
            .map(|value| value.value().map(|value| value.to_vec()))
            .collect::<Result<Vec<Vec<u8>>, fjall::Error>>()?;

        let mut outcome = SignedMessages::default();

        values.into_iter().try_for_each(|value| {
            let message = bitcode::decode::<SignedMessageData>(&value)
                .or(Err(KrillError::UnableToDeserializeSignedMessages))?;

            outcome.insert(message.message_hash, message);

            Ok::<(), KrillError>(())
        })?;

        Ok(outcome)
    }

    async fn clear_participant_messages(
        &self,
        message_hash: &Message32ByteHash,
    ) -> KrillResult<()> {
        let keyspace = self.participant_messages_keyspace().await?;

        keyspace
            .remove(message_hash)
            .or(Err(KrillError::UnableToRemoveValidSignedParticipantMessage))
    }
}
