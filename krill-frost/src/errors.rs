use std::io::ErrorKind;

use crate::{Message32ByteHash, SigningState, StoreKeyspace};

pub type KrillResult<T> = Result<T, KrillError>;

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum KrillError {
    #[error("Unable to locate the home directory")]
    UnableToFindHomeDirectory,
    #[error("The home directory path is not valid UTF-8")]
    HomeDirPathNotUtf8,
    #[error("Global storage is not initialized yet it is being called")]
    GlobalStorageNotInitialized,
    #[error("The maximum number of signers must be equal to or greater than the minimum number of signers")]
    MinimumSignersGreaterThanMaximumSigners,
    #[error("Unable to derive a DKG Identifier from random bytes")]
    IdentifierDerivationNotSupported,
    #[error("Frost Identifier already exists in storage")]
    IdentifierAlreadyExists,
    #[error("The FROST DKG Identifier Secret was not found")]
    FrostIdentifierNotFound,
    #[error("The FROST DKG Round1 Secret was not found")]
    Round1SecretNotFound,
    #[error("The FROST DKG Round1 Secret was not found")]
    Part1PublicPackageNotFound,
    #[error("The FROST DKG Round2 Secret was not found")]
    Part2SecretNotFound,
    #[error("The FROST Signing Round 1 Nonces are not found for the message")]
    Round1NoncesNotFound,
    #[error("The FROST Signing Round 1 Commitments are not found for the message")]
    Round1CommitmentsNotFound,
    #[error("There must be at least two signers for perform distributed key generation")]
    ThereMustBeAtLeast2Signers,
    #[error("Attempted to transition FROST DKG state yet the state is already finalized")]
    DkgStateAlreadyFinalized,
    #[error("Invalid FROST DKG state. Error: `{0}`.")]
    InvalidDkgState(&'static str),
    #[error("Unable to perform key generation for part 1. Error: `{0}`")]
    Part1KeyGenerationError(String),
    #[error("The maximum number of parties has been reached yet more part1 packages have been received.")]
    Part1MaximumPartiesReached,
    #[error("The maximum number of parties has been reached yet more part2 packages have been received.")]
    Part2MaximumPartiesReached,
    #[error("Current FROST DKG state is `{0}` yet FROST DKG state is supposed to be Part 2")]
    InvalidFrostDkgState(String),
    #[error("Unable to perform key generation for part 2. Error: `{0}`.")]
    Part2KeyGenerationError(String),
    #[error("Unable to finalize FROST DKG part3. Error: `{0}`.")]
    Part3Finalize(String),
    #[error("Unable to create a fixed size byte array from a slice. Err: `{0}`.")]
    ToByteArray(&'static str),
    #[error("Unable to deserialize PublicPackage. Error: `{0}`.")]
    DeserializePublicPackage(String),
    #[error("The message was not found in the list of message to perform distributed signing on.")]
    MessageToSignNotFound,
    #[error("The Signing Package for the message was not found. Has signing round 1 been done?")]
    SigningPackageNotFound,
    #[error("The round2 signing share was not found")]
    SignatureShareNotFound,
    #[error("The outcome signature of distributed signing was not found")]
    AggregateSignatureNotFound,
    #[error("The message hash that was provided is invalid")]
    InvalidMessageToSign,
    #[error("Frost distributed signing Round1 error. Error: `{0}`.")]
    SigningRound1(String),
    #[error("Round2 FROST distributed signing error. Error: `{0}`.")]
    SigningRound2(String),
    #[error("Encountered invalid participants: {as_hexes:?}", as_hexes = .0.iter().map(|value| 
    {
        bytes_as_hex(value)
    }).collect::<Vec<String>>())]
    InvalidParticipants(Vec<Vec<u8>>),
    #[error("The current state of signing message `{message_hash}` is `{state}` instead of SigningState::Round1", message_hash = bytes_as_hex(.message_hash))]
    ExpectedRound1SigningState {
        message_hash: Message32ByteHash,
        state: SigningState,
    },
    #[error("The current state of signing message `{message_hash}` is `{state}` instead of SigningState::Round2", message_hash = bytes_as_hex(.message_hash))]
    ExpectedRound2SigningState {
        message_hash: Message32ByteHash,
        state: SigningState,
    },
    #[error("The current state of signing message `{message_hash}` is `{state}` instead of SigningState::Aggregate", message_hash = bytes_as_hex(.message_hash))]
    ExpectedAggregateSigningState {
        message_hash: Message32ByteHash,
        state: SigningState,
    },
    #[error("A participant that is not allowed to sign this message  `{message_hash}`  tried to commit to the first round of signing the message. The offending participant ID is `{identifier}`.", message_hash = bytes_as_hex(.message_hash), identifier = bytes_as_hex(.participant))]
    InvalidParticipant {
        message_hash: Message32ByteHash,
        participant: Vec<u8>,
    },
    #[error("The participant of Round2 signing has no Signing nonces or signing commitments. Try doing round1 first")]
    SigningRound1NoncesAndCommitmentsNotFound,
    #[error("Unable to aggregate the signatures. Error: `{0}`!")]
    UnableToAggregateSignature(String),
    #[error("The aggregate signature is valid but was unable to remove it from storage")]
    UnableToRemoveValidSignedParticipantMessage,
    #[error("The group signature verification failed for the given message. Error: `{0}`!")]
    InvalidAggregateSignature(String),
    #[error("Unable to deserialize bytes into FrostDkgData struct.")]
    UnableToDeserializeFrostDkgData,
    #[error("Encountered I/O error: `{0}`")]
    Io(ErrorKind),
    #[error("The storage encountered an error: `{0}`!")]
    Store(String),
    #[error("The `{key}` for the FROST Signing Keypair was not found in the store", key = StoreKeyspace::FrostKeypair.to_str())]
    FrostKeypairKeyspaceNotFound,
    #[error("The FROST Keypair key was not found in storage")]
    FrostKeypairDataNotFound,
    #[error("Unable to deserialize the FROST keypair bytes from storage")]
    UnableToDeserializeFrostKeypairData,
    #[error("The Coordinator signing requests key was not found in storage")]
    CoordinatorDataNotFound,
    #[error("Unable to deserialize the coordinator signing requests bytes from storage")]
    UnableToDeserializeCoordinatorMessages,
    #[error("The Participants signing requests key was not found in storage")]
    ParticipantMessagesDataNotFound,
    #[error("Unable to deserialize the participants signing requests bytes from storage")]
    UnableToDeserializeParticipantMessages,
    #[error("The signed messages key was not found in storage")]
    SignedMessagesDataNotFound,
    #[error("Unable to deserialize the signed messages bytes from storage")]
    UnableToDeserializeSignedMessages,
    #[error("Unable to deserialize bytes into FROST Identifier")]
    UnableToDeserializeFrostIdentifier,
    #[error("Unable to serialize FROST Round1 SecretPackage")]
    UnableToSerializeFrostDkgRound1SecretPackage,
    #[error("Unable to deserialize FROST Round1 SecretPackage")]
    UnableToDeserializeFrostDkgRound1SecretPackage,
    #[error("Unable to serialize FROST Round1 PublicPackage")]
    UnableToSerializeFrostDkgRound1PublicPackage,
    #[error("Unable to deserialize FROST Round1 PublicPackage")]
    UnableToDeserializeFrostDkgRound1PublicPackage,
    #[error("Unable to serialize FROST Round2 SecretPackage")]
    UnableToSerializeFrostDkgRound2SecretPackage,
    #[error("Unable to deserialize FROST Round2 SecretPackage")]
    UnableToDeserializeFrostDkgRound2SecretPackage,
    #[error("Unable to serialize FROST Round2 Public Package")]
    UnableToSerializeFrostDkgRound2PublicPackage,
    #[error("Unable to deserialize FROST Round2 Public Package")]
    UnableToDeserializeFrostDkgRound2PublicPackage,
    #[error("Unable to serialize FROST Signing KeyPackage")]
    UnableToSerializeFrostSigningKeyPackage,
    #[error("Unable to deserialize FROST Signing KeyPackage")]
    UnableToDeserializeFrostSigningKeyPackage,
    #[error("Unable to serialize FROST Signing PublicKeyPackage")]
    UnableToSerializeFrostSigningPublicKeyPackage,
    #[error("Unable to deserialize FROST Signing PublicKeyPackage")]
    UnableToDeserializeFrostSigningPublicKeyPackage,
    #[error("Unable to serialize FROST SigningNonces")]
    UnableToSerializeFrostSigningNonces,
    #[error("Unable to deserialize FROST SigningNonces")]
    UnableToDeserializeFrostSigningNonces,
    #[error("Unable to serialize FROST SigningCommitments")]
    UnableToSerializeFrostSigningCommitments,
    #[error("Unable to deserialize FROST SigningCommitments")]
    UnableToDeserializeFrostSigningCommitments,
    #[error("Unable to serialize FROST SigningPackage")]
    UnableToSerializeFrostSigningPackage,
    #[error("Unable to deserialize FROST SigningPackage")]
    UnableToDeserializeFrostSigningPackage,
    #[error("Unable to serialize FROST SignatureShare")]
    UnableToSerializeFrostSignature,
    #[error("Unable to deserialize FROST SignatureShare")]
    UnableToDeserializeFrostSignatureShare,
    #[error("Unable to deserialize FROST Signature")]
    UnableToDeserializeFrostSignature,
    #[error("Unable to deserialize bytes into ParticipantMessageData")]
    UnableToDeserializeParticipantMessageData,
    #[error("Unable to deserialize bytes into CoordinatorDataNotFound")]
    UnableToDeserializeCoordinatorDataNotFound,
    #[error("Unable to deserialize bytes into SignedMessagesDataNotFound")]
    UnableToDeserializeSignedMessagesDataNotFound,
}

fn bytes_as_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{:0x?} ", byte))
        .collect::<String>()
        .trim()
        .to_string()
}

impl From<fjall::Error> for KrillError {
    fn from(error: fjall::Error) -> Self {
        match error {
            fjall::Error::Io(io_error) => Self::Io(io_error.kind()),
            _ => Self::Store(error.to_string()),
        }
    }
}
