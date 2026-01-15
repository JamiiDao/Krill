use std::{collections::BTreeMap, future::Future};

use frost_core::Ciphersuite;

use crate::{
    CoordinatorMessageData, CoordinatorMessages, FrostDkgData, FrostDkgState, FrostIdentifier,
    FrostKeypairData, KrillResult, Message32ByteHash, ParticipantMessageData, ParticipantMessages,
    SignedMessageData, SignedMessages,
};

pub trait FrostStorage<C: Ciphersuite> {
    fn get_identifier(&self) -> impl Future<Output = KrillResult<FrostIdentifier>>;

    fn set_identifier(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn set_keypair_data(
        &self,
        frost_keypair_data: &FrostKeypairData,
    ) -> impl Future<Output = KrillResult<()>>;

    fn set_coordinator_message(
        &self,
        message: &CoordinatorMessageData,
    ) -> impl Future<Output = KrillResult<()>>;

    fn set_participant_message(
        &self,
        message: &ParticipantMessageData,
    ) -> impl Future<Output = KrillResult<()>>;

    fn set_signed_message(
        &self,
        signed_message_data: &SignedMessageData,
    ) -> impl Future<Output = KrillResult<()>>;

    fn get_keypair_data(&self) -> impl Future<Output = KrillResult<FrostKeypairData>>;

    fn get_coordinator_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> impl Future<Output = KrillResult<CoordinatorMessageData>>;

    fn get_participant_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> impl Future<Output = KrillResult<ParticipantMessageData>>;

    fn get_signed_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> impl Future<Output = KrillResult<SignedMessageData>>;

    fn get_coordinator_messages(&self) -> impl Future<Output = KrillResult<CoordinatorMessages>>;

    fn get_participant_messages(&self) -> impl Future<Output = KrillResult<ParticipantMessages>>;

    fn get_signed_messages(&self) -> impl Future<Output = KrillResult<SignedMessages>>;

    fn is_valid_participant(
        &self,
        participant: &frost_core::Identifier<C>,
        frost_keypair_data: &FrostKeypairData,
    ) -> bool;

    fn clear_participant_messages(
        &self,
        message_hash: &Message32ByteHash,
    ) -> impl Future<Output = KrillResult<()>>;

    fn serialize(&self, data: &FrostDkgData) -> Vec<u8>;

    fn deserialize(&self, bytes: &[u8]) -> KrillResult<FrostDkgData>;

    fn set_state(&self, state: FrostDkgState) -> impl Future<Output = KrillResult<()>>;

    fn get_state(&self) -> impl Future<Output = KrillResult<FrostDkgState>>;

    fn set_maximum_signers(&self, maximum_signers: u16) -> impl Future<Output = KrillResult<()>>;

    fn get_maximum_signers(&self) -> impl Future<Output = KrillResult<u16>>;

    fn set_minimum_signers(&self, minimum_signers: u16) -> impl Future<Output = KrillResult<()>>;

    fn get_minimum_signers(&self) -> impl Future<Output = KrillResult<u16>>;

    fn set_part1_package(
        &self,
        secret: frost_core::keys::dkg::round1::SecretPackage<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn get_part1_secret_package(
        &self,
    ) -> impl Future<Output = KrillResult<frost_core::keys::dkg::round1::SecretPackage<C>>>;

    fn get_part1_public_package(
        &self,
    ) -> impl Future<Output = KrillResult<frost_core::keys::dkg::round1::Package<C>>>;

    fn add_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn has_part1_received_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<bool>>;

    fn get_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<Option<frost_core::keys::dkg::round1::Package<C>>>>;

    fn get_all_part1_received_packages(
        &self,
    ) -> impl Future<
        Output = KrillResult<
            BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>,
        >,
    >;

    fn part1_received_packages_count(&self) -> impl Future<Output = KrillResult<usize>>;

    fn part2_received_packages_count(&self) -> impl Future<Output = KrillResult<usize>>;

    fn set_part2_package(
        &self,
        secret: frost_core::keys::dkg::round2::SecretPackage<C>,
        packages: BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn add_part2_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round2::Package<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn get_part2_secret(
        &self,
    ) -> impl Future<Output = KrillResult<frost_core::keys::dkg::round2::SecretPackage<C>>>;

    fn get_part2_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<Option<frost_core::keys::dkg::round2::Package<C>>>>;

    fn get_all_part2_received_packages(
        &self,
    ) -> impl Future<
        Output = KrillResult<
            BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
        >,
    >;

    fn clear_dkg_data(&self) -> impl Future<Output = KrillResult<()>>;
}
