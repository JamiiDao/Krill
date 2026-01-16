use std::collections::BTreeMap;

use frost_core::Ciphersuite;
use krill_common::{KrillError, KrillResult};

use crate::{
    CoordinatorMessageData, CoordinatorMessages, FrostDkgData, FrostDkgState, FrostIdentifier,
    FrostKeypairData, FrostRound1PublicPackage, FrostRound1SecretPackage, FrostRound2PublicPackage,
    FrostRound2SecretPackage, FrostStorage, FrostStore, Message32ByteHash, ParticipantMessageData,
    ParticipantMessages, SignedMessageData, SignedMessages, StoreKeys,
};

impl<C: Ciphersuite + Send + Sync + Clone> FrostStorage<C> for FrostStore<C> {
    async fn set_identifier(&self, identifier: &frost_core::Identifier<C>) -> KrillResult<()> {
        let identifier = FrostIdentifier::encode(identifier);
        let mut frost_keypair_data = self.get_keypair_data().await?;
        frost_keypair_data.identifier = identifier;

        self.set_keypair_data(&frost_keypair_data).await
    }

    async fn get_identifier(&self) -> KrillResult<FrostIdentifier> {
        Ok(self.get_keypair_data().await?.identifier)
    }

    async fn set_keypair_data(&self, frost_keypair_data: &FrostKeypairData) -> KrillResult<()> {
        let frost_keypair_bytes = bitcode::encode(frost_keypair_data);
        self.set_dkg_op(StoreKeys::KeypairData, frost_keypair_bytes)
            .await
    }

    async fn set_coordinator_message(&self, message: &CoordinatorMessageData) -> KrillResult<()> {
        let message_bytes = bitcode::encode(message);

        let keyspace = self.coordinator_messages_keyspace();

        self.set_op(keyspace, message.message_hash, message_bytes)
            .await
    }

    async fn set_participant_message(&self, message: &ParticipantMessageData) -> KrillResult<()> {
        let message_bytes = bitcode::encode(message);

        let keyspace = self.participant_messages_keyspace();

        self.set_op(keyspace, message.message_hash, message_bytes)
            .await
    }

    async fn set_signed_message(&self, signed_message_data: &SignedMessageData) -> KrillResult<()> {
        let message_bytes = bitcode::encode(signed_message_data);
        let keyspace = self.signed_messages_keyspace();

        self.set_op(keyspace, signed_message_data.message_hash, message_bytes)
            .await
    }

    async fn get_keypair_data(&self) -> KrillResult<FrostKeypairData> {
        let keyspace = self.keypair_keyspace();

        let data_bytes = blocking::unblock(move || keyspace.get(StoreKeys::KeypairData.to_str()))
            .await?
            .map(|data| data.to_vec())
            .ok_or(KrillError::FrostKeypairDataNotFound)?;

        bitcode::decode(&data_bytes).or(Err(KrillError::UnableToDeserializeFrostKeypairData))
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
        let keyspace = self.coordinator_messages_keyspace();

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
        let keyspace = self.coordinator_messages_keyspace();

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
        let keyspace = self.signed_messages_keyspace();

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
        let keyspace = self.participant_messages_keyspace();

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
        let keyspace = self.participant_messages_keyspace();

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
        let keyspace = self.participant_messages_keyspace();

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
        let keyspace = self.participant_messages_keyspace();

        keyspace
            .remove(message_hash)
            .or(Err(KrillError::UnableToRemoveValidSignedParticipantMessage))
    }

    fn serialize(&self, data: &FrostDkgData) -> Vec<u8> {
        bitcode::encode(data)
    }

    fn deserialize(&self, bytes: &[u8]) -> KrillResult<FrostDkgData> {
        bitcode::decode(bytes).or(Err(KrillError::UnableToDeserializeFrostDkgData))
    }

    async fn set_state(&self, dkg_state: FrostDkgState) -> KrillResult<()> {
        let mut data = self.get_and_deserialize_dkg_data().await?;
        data.dkg_state = dkg_state;

        let data_as_bytes = self.serialize(&data);

        self.set_dkg_op(StoreKeys::Dkg, data_as_bytes).await
    }

    async fn get_state(&self) -> KrillResult<FrostDkgState> {
        Ok(self.get_and_deserialize_dkg_data().await?.dkg_state)
    }

    async fn set_maximum_signers(&self, maximum_signers: u16) -> KrillResult<()> {
        let mut data = self.get_and_deserialize_dkg_data().await?;
        data.maximum_signers = maximum_signers;

        let data_as_bytes = self.serialize(&data);

        self.set_dkg_op(StoreKeys::Dkg, data_as_bytes).await
    }

    async fn get_maximum_signers(&self) -> KrillResult<u16> {
        Ok(self.get_and_deserialize_dkg_data().await?.maximum_signers)
    }

    async fn set_minimum_signers(&self, minimum_signers: u16) -> KrillResult<()> {
        let mut data = self.get_and_deserialize_dkg_data().await?;
        data.minimum_signers = minimum_signers;

        let data_as_bytes = self.serialize(&data);

        self.set_dkg_op(StoreKeys::Dkg, data_as_bytes).await
    }

    async fn get_minimum_signers(&self) -> KrillResult<u16> {
        Ok(self.get_and_deserialize_dkg_data().await?.minimum_signers)
    }

    async fn set_part1_package(
        &self,
        secret: frost_core::keys::dkg::round1::SecretPackage<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> KrillResult<()> {
        let mut data = self.get_and_deserialize_dkg_data().await?;
        data.part1_secret
            .replace(FrostRound1SecretPackage::encode(&secret)?);
        data.part1_package
            .replace(FrostRound1PublicPackage::encode(&package)?);

        let data_as_bytes = self.serialize(&data);

        self.set_dkg_op(StoreKeys::Dkg, data_as_bytes).await
    }

    /// This zeroizes the secret so its only accessible once
    async fn get_part1_secret_package(
        &self,
    ) -> KrillResult<frost_core::keys::dkg::round1::SecretPackage<C>> {
        self.get_and_deserialize_dkg_data()
            .await?
            .part1_secret
            .as_ref()
            .map(|value| value.decode::<C>())
            .transpose()?
            .take()
            .ok_or(KrillError::Round1SecretNotFound)
    }

    async fn get_part1_public_package(
        &self,
    ) -> KrillResult<frost_core::keys::dkg::round1::Package<C>> {
        self.get_and_deserialize_dkg_data()
            .await?
            .part1_package
            .as_ref()
            .map(|value| value.decode::<C>())
            .transpose()?
            .clone()
            .ok_or(KrillError::Part1PublicPackageNotFound)
    }

    async fn add_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> KrillResult<()> {
        let mut data = self.get_and_deserialize_dkg_data().await?;
        data.received_part1_packages.insert(
            FrostIdentifier::encode(&identifier),
            FrostRound1PublicPackage::encode(&package)?,
        );

        let data_as_bytes = self.serialize(&data);

        self.set_dkg_op(StoreKeys::Dkg, data_as_bytes).await
    }

    async fn get_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> KrillResult<Option<frost_core::keys::dkg::round1::Package<C>>> {
        self.get_and_deserialize_dkg_data()
            .await?
            .received_part1_packages
            .get(&FrostIdentifier::encode(&identifier))
            .map(|value| value.decode::<C>())
            .transpose()
    }

    async fn has_part1_received_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> KrillResult<bool> {
        Ok(self
            .get_and_deserialize_dkg_data()
            .await?
            .received_part1_packages
            .contains_key(&FrostIdentifier::encode(identifier)))
    }

    async fn get_all_part1_received_packages(
        &self,
    ) -> KrillResult<BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>>
    {
        let packages = self
            .get_and_deserialize_dkg_data()
            .await?
            .received_part1_packages;

        packages
            .iter()
            .map(|(key, value)| {
                let identifier = key.decode::<C>()?;
                let public_package = value.decode::<C>()?;

                Ok((identifier, public_package))
            })
            .collect::<Result<
                BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>,
                _,
            >>()
    }

    async fn part1_received_packages_count(&self) -> KrillResult<usize> {
        Ok(self
            .get_and_deserialize_dkg_data()
            .await?
            .received_part1_packages
            .len())
    }

    async fn part2_received_packages_count(&self) -> KrillResult<usize> {
        Ok(self
            .get_and_deserialize_dkg_data()
            .await?
            .received_part2_packages
            .len())
    }

    async fn set_part2_package(
        &self,
        secret: frost_core::keys::dkg::round2::SecretPackage<C>,
        packages: BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    ) -> KrillResult<()> {
        let mut data = self.get_and_deserialize_dkg_data().await?;
        data.part2_secret
            .replace(FrostRound2SecretPackage::encode(&secret)?);
        let packages = packages
            .iter()
            .map(|(key, value)| {
                let identifier = FrostIdentifier::encode::<C>(key);
                let packages = FrostRound2PublicPackage::encode::<C>(value)?;

                Ok::<_, KrillError>((identifier, packages))
            })
            .collect::<Result<BTreeMap<FrostIdentifier, FrostRound2PublicPackage>, _>>()?;
        data.part2_package = packages;

        let data_as_bytes = self.serialize(&data);

        self.set_dkg_op(StoreKeys::Dkg, data_as_bytes).await
    }

    async fn get_part2_secret(
        &self,
    ) -> KrillResult<frost_core::keys::dkg::round2::SecretPackage<C>> {
        self.get_and_deserialize_dkg_data()
            .await?
            .part2_secret
            .as_ref()
            .map(|value| value.decode::<C>())
            .transpose()?
            .ok_or(KrillError::Part2SecretNotFound)
    }

    async fn get_part2_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> KrillResult<Option<frost_core::keys::dkg::round2::Package<C>>> {
        self.get_and_deserialize_dkg_data()
            .await?
            .part2_package
            .get(&FrostIdentifier::encode(identifier))
            .map(|value| value.decode::<C>())
            .transpose()
    }

    async fn get_all_part2_received_packages(
        &self,
    ) -> KrillResult<BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>>
    {
        self.get_and_deserialize_dkg_data()
            .await?
            .received_part2_packages
            .iter()
            .map(|(key, value)| {
                let identifier = key.decode::<C>()?;
                let package = value.decode::<C>()?;

                Ok::<_, KrillError>((identifier, package))
            })
            .collect()
    }

    async fn add_part2_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round2::Package<C>,
    ) -> KrillResult<()> {
        let mut data = self.get_and_deserialize_dkg_data().await?;
        data.received_part2_packages.insert(
            FrostIdentifier::encode(&identifier),
            FrostRound2PublicPackage::encode(&package)?,
        );

        let data_as_bytes = self.serialize(&data);

        self.set_dkg_op(StoreKeys::Dkg, data_as_bytes).await
    }

    async fn clear_dkg_data(&self) -> KrillResult<()> {
        let keyspace = self.keypair_keyspace();

        keyspace
            .remove(StoreKeys::Dkg.to_str())
            .map_err(|error| KrillError::Store(error.to_string()))?;

        Ok(())
    }
}
