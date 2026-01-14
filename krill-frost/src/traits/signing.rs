use core::fmt;
use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
};

use bitcode::{Decode, Encode};
use frost_core::Ciphersuite;

use crate::{
    FrostIdentifier, FrostSignature, FrostSignatureShare, FrostSigningCommitments,
    FrostSigningKeyPackage, FrostSigningNonces, FrostSigningPackage, FrostSigningPublicKeyPackage,
    KrillResult,
};

pub type Message32ByteHash = [u8; 32];

pub trait FrostDistributedSigning {
    type DkgCipherSuite: Ciphersuite;

    fn storage(&self) -> impl FrostDistributedSigningOps<Self::DkgCipherSuite>;

    fn signal_round1(
        &self,
        message: Message32ByteHash,
        participants: &[frost_core::Identifier<Self::DkgCipherSuite>],
        is_signer: bool,
    ) -> impl Future<Output = KrillResult<SigningRound1RequestData>>;

    fn round1_commit(
        &self,
        message: SigningRound1RequestData,
    ) -> impl Future<Output = KrillResult<Round1CommitData>>;

    fn receive_round1_commit(
        &self,
        commit_data: Round1CommitData,
    ) -> impl Future<Output = KrillResult<SigningState>>;

    fn signing_package(
        &self,
        message: &Message32ByteHash,
        is_signer: bool,
    ) -> impl Future<Output = KrillResult<SigningPackageData>>;

    fn round2_commit(
        &self,
        signing_package_data: SigningPackageData,
    ) -> impl Future<Output = KrillResult<Round2SigningData>>;

    fn receive_round2_commit(
        &self,
        message_data: Round2SigningData,
    ) -> impl Future<Output = KrillResult<SigningState>>;

    fn aggregate(
        &self,
        message_hash: Message32ByteHash,
    ) -> impl Future<Output = KrillResult<AggregateSignatureData>>;

    fn verify(
        &self,
        aggregate_signature_data: &AggregateSignatureData,
    ) -> impl Future<Output = KrillResult<()>>;

    fn verify_and_remove(
        &self,
        aggregate_signature_data: &AggregateSignatureData,
    ) -> impl Future<Output = KrillResult<()>>;
}

pub trait FrostDistributedSigningOps<C: Ciphersuite> {
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

    fn get_identifier(&self) -> impl Future<Output = KrillResult<frost_core::Identifier<C>>>;

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
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct FrostKeypairData {
    pub identifier: FrostIdentifier,
    pub maximum_signers: u16,
    pub minimum_signers: u16,
    pub secret: FrostSigningKeyPackage,
    pub public_package: FrostSigningPublicKeyPackage,
    pub participants: Vec<FrostIdentifier>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct SignedMessageData {
    pub participants: Vec<FrostIdentifier>,
    pub message_hash: Message32ByteHash,
    pub signature: FrostSignature,
    pub public_key_package: FrostSigningPublicKeyPackage,
}

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct CoordinatorMessageData {
    pub message_hash: Message32ByteHash,
    pub is_signer: bool,
    pub state: SigningState,
    pub participants: Vec<FrostIdentifier>,
    pub nonces: Option<FrostSigningNonces>,
    pub signing_package: Option<FrostSigningPackage>,
    pub commitments: BTreeMap<FrostIdentifier, FrostSigningCommitments>,
    pub signature_shares: BTreeMap<FrostIdentifier, FrostSignatureShare>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct ParticipantMessageData {
    pub message_hash: Message32ByteHash,
    pub participants: Vec<FrostIdentifier>,
    pub coordinator: FrostIdentifier,
    pub round1: Option<(FrostSigningNonces, FrostSigningCommitments)>,
    pub signing_package: Option<FrostSigningPackage>,
    pub round2: Option<FrostSignatureShare>,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Encode, Decode)]
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

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Encode, Decode)]
pub enum SigningInstruction {
    #[default]
    Round1,
    Round2,
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct SigningRound1RequestData {
    pub message_hash: Message32ByteHash,
    pub participants: Vec<FrostIdentifier>,
    pub coordinator: FrostIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct Round1CommitData {
    pub message_hash: Message32ByteHash,
    pub identifier: FrostIdentifier,
    pub commitments: FrostSigningCommitments,
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct SigningPackageData {
    pub message_hash: Message32ByteHash,
    pub signing_package: FrostSigningPackage,
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct Round2SigningData {
    pub message_hash: Message32ByteHash,
    pub identifier: FrostIdentifier,
    pub signature_share: FrostSignatureShare,
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct AggregateSignatureData {
    pub message_hash: Message32ByteHash,
    pub aggregate_signature: FrostSignature,
    pub coordinator: FrostIdentifier,
    pub participants: Vec<FrostIdentifier>,
}

pub type CoordinatorMessages = BTreeMap<Message32ByteHash, CoordinatorMessageData>; // Bytes for CoordinatorMessageData
pub type ParticipantMessages = HashMap<Message32ByteHash, ParticipantMessageData>; // Bytes for participant MessageData
pub type SignedMessages = HashMap<Message32ByteHash, SignedMessageData>; // Bytes for SignedMessageData
