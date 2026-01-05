use core::fmt;
use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
};

use async_lock::{RwLockReadGuard, RwLockWriteGuard};
use frost_core::{
    keys::PublicKeyPackage, round1::SigningCommitments, round2::SignatureShare, Ciphersuite,
    Signature, SigningPackage,
};
use serde::{Deserialize, Serialize};

pub type Message32ByteHash = [u8; 32];

pub trait FrostDistributedSigning {
    type DkgGenericError: core::error::Error;
    type DkgCipherSuite: Ciphersuite;

    fn storage(
        &self,
    ) -> impl Future<
        Output = Result<
            impl FrostDistributedSigningStorage<Self::DkgCipherSuite, Self::DkgGenericError>,
            Self::DkgGenericError,
        >,
    >;

    fn identifier(
        &self,
    ) -> impl Future<Output = Result<frost_core::Identifier<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn signal_round1(
        &self,
        message: Message32ByteHash,
        participants: &[frost_core::Identifier<Self::DkgCipherSuite>],
        is_signer: bool,
    ) -> impl Future<
        Output = Result<SigningRound1RequestData<Self::DkgCipherSuite>, Self::DkgGenericError>,
    >;

    fn round1_commit(
        &self,
        message: SigningRound1RequestData<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<Round1CommitData<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn receive_round1_commit(
        &self,
        commit_data: Round1CommitData<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<SigningState, Self::DkgGenericError>>;

    fn signing_package(
        &self,
        message: &Message32ByteHash,
        is_signer: bool,
    ) -> impl Future<Output = Result<SigningPackageData<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn round2_commit(
        &self,
        signing_package_data: SigningPackageData<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<Round2SigningData<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn receive_round2_commit(
        &self,
        message_data: Round2SigningData<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<SigningState, Self::DkgGenericError>>;

    fn aggregate(
        &self,
        message_hash: Message32ByteHash,
    ) -> impl Future<Output = Result<AggregateSignatureData<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn verify(
        &self,
        aggregate_signature_data: &AggregateSignatureData<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<(), Self::DkgGenericError>>;

    fn verify_and_remove(
        &self,
        aggregate_signature_data: &AggregateSignatureData<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<(), Self::DkgGenericError>>;

    fn all_coordinator_messages(
        &self,
    ) -> impl Future<
        Output = Result<
            BTreeMap<Message32ByteHash, MessageData<Self::DkgCipherSuite>>,
            Self::DkgGenericError,
        >,
    >;

    fn all_participant_messages(
        &self,
    ) -> impl Future<
        Output = Result<
            HashMap<Message32ByteHash, ParticipantMessageData<Self::DkgCipherSuite>>,
            Self::DkgGenericError,
        >,
    >;

    fn get_coordinator_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> impl Future<Output = Result<Option<MessageData<Self::DkgCipherSuite>>, Self::DkgGenericError>>;

    fn get_participant_messages(
        &self,
        message_hash: &Message32ByteHash,
    ) -> impl Future<
        Output = Result<
            Option<ParticipantMessageData<Self::DkgCipherSuite>>,
            Self::DkgGenericError,
        >,
    >;
}

pub trait FrostDistributedSigningStorage<C: Ciphersuite, E: core::error::Error> {
    fn get(&self) -> impl Future<Output = Result<RwLockReadGuard<'_, impl FrostKeyStore<C>>, E>>;

    fn set(&self) -> impl Future<Output = Result<RwLockWriteGuard<'_, impl FrostKeyStore<C>>, E>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FrostSigningData<C: Ciphersuite> {
    pub identifier: frost_core::Identifier<C>,
    pub maximum_signers: u16,
    pub minimum_signers: u16,
    pub secret: frost_core::keys::KeyPackage<C>,
    pub public_package: frost_core::keys::PublicKeyPackage<C>,
    pub participants: Vec<frost_core::Identifier<C>>,
}

impl<C: Ciphersuite> FrostSigningData<C> {
    pub fn identifier(&self) -> &frost_core::Identifier<C> {
        &self.identifier
    }

    pub fn maximum_signers(&self) -> u16 {
        self.maximum_signers
    }

    pub fn minimum_signers(&self) -> u16 {
        self.minimum_signers
    }

    pub fn secret(&self) -> &frost_core::keys::KeyPackage<C> {
        &self.secret
    }

    pub fn public_package(&self) -> &frost_core::keys::PublicKeyPackage<C> {
        &self.public_package
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedMessageData<C: Ciphersuite> {
    pub participants: Vec<frost_core::Identifier<C>>,
    pub message_hash: Message32ByteHash,
    pub signature: Signature<C>,
    pub public_key_package: PublicKeyPackage<C>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct MessageData<C: Ciphersuite> {
    pub is_signer: bool,
    pub state: SigningState,
    pub participants: Vec<frost_core::Identifier<C>>,
    pub message_hash: Message32ByteHash,
    pub nonces: Option<frost_core::round1::SigningNonces<C>>,
    pub signing_package: Option<SigningPackage<C>>,
    pub commitments: BTreeMap<frost_core::Identifier<C>, frost_core::round1::SigningCommitments<C>>,
    pub signature_shares: BTreeMap<frost_core::Identifier<C>, SignatureShare<C>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParticipantMessageData<C: Ciphersuite> {
    pub participants: Vec<frost_core::Identifier<C>>,
    pub coordinator: frost_core::Identifier<C>,
    pub round1: Option<(
        frost_core::round1::SigningNonces<C>,
        frost_core::round1::SigningCommitments<C>,
    )>,
    pub signing_package: Option<SigningPackage<C>>,
    pub round2: Option<frost_core::round2::SignatureShare<C>>,
}

pub trait FrostKeyStore<C: Ciphersuite> {
    fn state(&self, message_hash: &Message32ByteHash) -> Option<SigningState> {
        self.get_message(message_hash)
            .map(|message_data| message_data.state)
    }

    fn keys_data(&self) -> &FrostSigningData<C>;

    fn get_message(&self, message_hash: &Message32ByteHash) -> Option<&MessageData<C>>;

    fn get_mut_message(&mut self, message_hash: &[u8; 32]) -> Option<&mut MessageData<C>>;

    fn coordinator_messages(&self) -> BTreeMap<[u8; 32], MessageData<C>>;

    fn participant_messages(&self) -> HashMap<[u8; 32], ParticipantMessageData<C>>;

    fn set_message(
        &mut self,
        message_hash: &Message32ByteHash,
        message_data: MessageData<C>,
    ) -> bool;

    fn remove_message(&mut self, message_hash: &Message32ByteHash) -> Option<MessageData<C>>;

    fn is_valid_participant(&self, participant: &frost_core::Identifier<C>) -> bool {
        self.keys_data()
            .participants
            .iter()
            .any(|identifier| identifier == participant)
    }

    fn set_participant_message(
        &mut self,
        message_hash: &Message32ByteHash,
        message_data: ParticipantMessageData<C>,
    ) -> bool;

    fn get_participant_message_mut(
        &mut self,
        message_hash: &Message32ByteHash,
    ) -> Option<&mut ParticipantMessageData<C>>;

    fn get_participant_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> Option<&ParticipantMessageData<C>>;

    fn remove_participant_message(&mut self, message_hash: &Message32ByteHash) -> bool;
}

#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize,
)]
pub enum SigningState {
    #[default]
    Round1,
    Round2,
    Aggregate,
}

impl fmt::Display for SigningState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize,
)]
pub enum SigningInstruction {
    #[default]
    Round1,
    Round2,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct SigningRound1RequestData<C: Ciphersuite> {
    pub message_hash: Message32ByteHash,
    pub participants: Vec<frost_core::Identifier<C>>,
    pub coordinator: frost_core::Identifier<C>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Round1CommitData<C: Ciphersuite> {
    pub message_hash: Message32ByteHash,
    pub identifier: frost_core::Identifier<C>,
    pub commitments: SigningCommitments<C>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct SigningPackageData<C: Ciphersuite> {
    pub message_hash: Message32ByteHash,
    pub signing_package: frost_core::SigningPackage<C>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Round2SigningData<C: Ciphersuite> {
    pub message_hash: Message32ByteHash,
    pub identifier: frost_core::Identifier<C>,
    pub signature_share: frost_core::round2::SignatureShare<C>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AggregateSignatureData<C: Ciphersuite> {
    pub message_hash: Message32ByteHash,
    pub aggregate_signature: Signature<C>,
    pub coordinator: frost_core::Identifier<C>,
    pub participants: Vec<frost_core::Identifier<C>>,
}
