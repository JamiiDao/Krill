use std::io::ErrorKind;

use crate::{KrillUtils, Message32ByteHash, SupportedLanguages};

pub type KrillResult<T> = Result<T, KrillError>;

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum KrillError {
    #[error("The server key was not found")]
    ServerSecretNotFound,
    #[error("Unable to set the static `SERVER_SECRET`")]
    UnableToSetServerSecret,
    #[error("The line `{0}` is not a valid `code = translation` entry. An example of a valid entry is `en = Hello World`.")]
    InvalidLanguageEntry(&'static str),
    #[error("Encountered an invalid BCP47 Code `{0}`.")]
    LanguageNotValidBcp47Code(&'static str),
    #[error("The translation for `{lang}` was not found.", lang = .0.as_str())]
    LanguageTranslationNotFound(SupportedLanguages),
    #[cfg(feature = "home-dir")]
    #[error("Unable to locate the home directory")]
    UnableToFindHomeDirectory,
    #[cfg(feature = "home-dir")]
    #[error("The home directory path is not valid UTF-8")]
    HomeDirPathNotUtf8,
    #[cfg(feature = "storage")]
    #[error("Global storage is not initialized yet it is being called")]
    GlobalStorageNotInitialized,
    #[cfg(feature = "storage")]
    #[error("The maximum number of signers must be equal to or greater than the minimum number of signers")]
    MinimumSignersGreaterThanMaximumSigners,
    #[cfg(feature = "storage")]
    #[error("Unable to derive a DKG Identifier from random bytes")]
    IdentifierDerivationNotSupported,
    #[cfg(feature = "storage")]
    #[error("Frost Identifier already exists in storage")]
    IdentifierAlreadyExists,
    #[cfg(feature = "storage")]
    #[error("The FROST DKG Identifier Secret was not found")]
    FrostIdentifierNotFound,
    #[cfg(feature = "storage")]
    #[error("The FROST DKG Round1 Secret was not found")]
    Round1SecretNotFound,
    #[cfg(feature = "storage")]
    #[error("The FROST DKG Round1 Secret was not found")]
    Part1PublicPackageNotFound,
    #[cfg(feature = "storage")]
    #[error("The FROST DKG Round2 Secret was not found")]
    Part2SecretNotFound,
    #[cfg(feature = "storage")]
    #[error("The FROST Signing Round 1 Nonces are not found for the message")]
    Round1NoncesNotFound,
    #[cfg(feature = "storage")]
    #[error("The FROST Signing Round 1 Commitments are not found for the message")]
    Round1CommitmentsNotFound,
    #[cfg(feature = "storage")]
    #[error("There must be at least two signers for perform distributed key generation")]
    ThereMustBeAtLeast2Signers,
    #[cfg(feature = "storage")]
    #[error("Attempted to transition FROST DKG state yet the state is already finalized")]
    DkgStateAlreadyFinalized,
    #[cfg(feature = "storage")]
    #[error("Invalid FROST DKG state. Error: `{0}`.")]
    InvalidDkgState(&'static str),
    #[cfg(feature = "storage")]
    #[error("Unable to perform key generation for part 1. Error: `{0}`")]
    Part1KeyGenerationError(String),
    #[cfg(feature = "storage")]
    #[error("The maximum number of parties has been reached yet more part1 packages have been received.")]
    Part1MaximumPartiesReached,
    #[cfg(feature = "storage")]
    #[error("The maximum number of parties has been reached yet more part2 packages have been received.")]
    Part2MaximumPartiesReached,
    #[cfg(feature = "storage")]
    #[error("Current FROST DKG state is `{0}` yet FROST DKG state is supposed to be Part 2")]
    InvalidFrostDkgState(String),
    #[cfg(feature = "storage")]
    #[error("Unable to perform key generation for part 2. Error: `{0}`.")]
    Part2KeyGenerationError(String),
    #[cfg(feature = "storage")]
    #[error("Unable to finalize FROST DKG part3. Error: `{0}`.")]
    Part3Finalize(String),
    #[cfg(feature = "storage")]
    #[error("Unable to create a fixed size byte array from a slice. Err: `{0}`.")]
    ToByteArray(&'static str),
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize PublicPackage. Error: `{0}`.")]
    DeserializePublicPackage(String),
    #[cfg(feature = "storage")]
    #[error("The message was not found in the list of message to perform distributed signing on.")]
    MessageToSignNotFound,
    #[cfg(feature = "storage")]
    #[error("The Signing Package for the message was not found. Has signing round 1 been done?")]
    SigningPackageNotFound,
    #[cfg(feature = "storage")]
    #[error("The round2 signing share was not found")]
    SignatureShareNotFound,
    #[cfg(feature = "storage")]
    #[error("The outcome signature of distributed signing was not found")]
    AggregateSignatureNotFound,
    #[cfg(feature = "storage")]
    #[error("The message hash that was provided is invalid")]
    InvalidMessageToSign,
    #[cfg(feature = "storage")]
    #[error("Frost distributed signing Round1 error. Error: `{0}`.")]
    SigningRound1(String),
    #[cfg(feature = "storage")]
    #[error("Round2 FROST distributed signing error. Error: `{0}`.")]
    SigningRound2(String),
    #[cfg(feature = "storage")]
    #[error("Encountered invalid participants: {as_hexes:?}", as_hexes = .0.iter().map(|value| 
    {
        KrillUtils::array_of_bytes_to_hex(value)
    }).collect::<Vec<String>>())]
    InvalidParticipants(Vec<Vec<u8>>),
    #[cfg(feature = "storage")]
    #[error("The current state of signing message `{message_hash}` is `{state}` instead of SigningState::Round1", message_hash = KrillUtils::array_of_bytes_to_hex(.message_hash))]
    ExpectedRound1SigningState {
        message_hash: Message32ByteHash,
        state: &'static str,
    },
    #[cfg(feature = "storage")]
    #[error("The current state of signing message `{message_hash}` is `{state}` instead of SigningState::Round2", message_hash = KrillUtils::array_of_bytes_to_hex(.message_hash))]
    ExpectedRound2SigningState {
        message_hash: Message32ByteHash,
        state: &'static str,
    },
    #[cfg(feature = "storage")]
    #[error("The current state of signing message `{message_hash}` is `{state}` instead of SigningState::Aggregate", message_hash = KrillUtils::array_of_bytes_to_hex(.message_hash))]
    ExpectedAggregateSigningState {
        message_hash: Message32ByteHash,
        state: &'static str,
    },
    #[cfg(feature = "storage")]
    #[error("A participant that is not allowed to sign this message  `{message_hash}`  tried to commit to the first round of signing the message. The offending participant ID is `{identifier}`.", message_hash = KrillUtils::array_of_bytes_to_hex(.message_hash), identifier = KrillUtils::array_of_bytes_to_hex(.participant))]
    InvalidParticipant {
        message_hash: Message32ByteHash,
        participant: Vec<u8>,
    },
    #[cfg(feature = "storage")]
    #[error("The participant of Round2 signing has no Signing nonces or signing commitments. Try doing round1 first")]
    SigningRound1NoncesAndCommitmentsNotFound,
    #[cfg(feature = "storage")]
    #[error("Unable to aggregate the signatures. Error: `{0}`!")]
    UnableToAggregateSignature(String),
    #[cfg(feature = "storage")]
    #[error("The aggregate signature is valid but was unable to remove it from storage")]
    UnableToRemoveValidSignedParticipantMessage,
    #[cfg(feature = "storage")]
    #[error("The group signature verification failed for the given message. Error: `{0}`!")]
    InvalidAggregateSignature(String),
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize bytes into FrostDkgData struct.")]
    UnableToDeserializeFrostDkgData,
    #[cfg(feature = "storage")]
    #[error("Encountered I/O error: `{0}`")]
    Io(ErrorKind),
    #[cfg(feature = "storage")]
    #[error("The storage encountered an error: `{0}`!")]
    Store(String),
    #[cfg(feature = "storage")]
    #[error("The `{0}` for the FROST Signing Keypair was not found in the store")]
    FrostKeypairKeyspaceNotFound(&'static str),
    #[cfg(feature = "storage")]
    #[error("The FROST Keypair key was not found in storage")]
    FrostKeypairDataNotFound,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize the FROST keypair bytes from storage")]
    UnableToDeserializeFrostKeypairData,
    #[cfg(feature = "storage")]
    #[error("The Coordinator signing requests key was not found in storage")]
    CoordinatorDataNotFound,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize the coordinator signing requests bytes from storage")]
    UnableToDeserializeCoordinatorMessages,
    #[cfg(feature = "storage")]
    #[error("The Participants signing requests key was not found in storage")]
    ParticipantMessagesDataNotFound,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize the participants signing requests bytes from storage")]
    UnableToDeserializeParticipantMessages,
    #[cfg(feature = "storage")]
    #[error("The signed messages key was not found in storage")]
    SignedMessagesDataNotFound,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize the signed messages bytes from storage")]
    UnableToDeserializeSignedMessages,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize bytes into FROST Identifier")]
    UnableToDeserializeFrostIdentifier,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST Round1 SecretPackage")]
    UnableToSerializeFrostDkgRound1SecretPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST Round1 SecretPackage")]
    UnableToDeserializeFrostDkgRound1SecretPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST Round1 PublicPackage")]
    UnableToSerializeFrostDkgRound1PublicPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST Round1 PublicPackage")]
    UnableToDeserializeFrostDkgRound1PublicPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST Round2 SecretPackage")]
    UnableToSerializeFrostDkgRound2SecretPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST Round2 SecretPackage")]
    UnableToDeserializeFrostDkgRound2SecretPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST Round2 Public Package")]
    UnableToSerializeFrostDkgRound2PublicPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST Round2 Public Package")]
    UnableToDeserializeFrostDkgRound2PublicPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST Signing KeyPackage")]
    UnableToSerializeFrostSigningKeyPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST Signing KeyPackage")]
    UnableToDeserializeFrostSigningKeyPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST Signing PublicKeyPackage")]
    UnableToSerializeFrostSigningPublicKeyPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST Signing PublicKeyPackage")]
    UnableToDeserializeFrostSigningPublicKeyPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST SigningNonces")]
    UnableToSerializeFrostSigningNonces,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST SigningNonces")]
    UnableToDeserializeFrostSigningNonces,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST SigningCommitments")]
    UnableToSerializeFrostSigningCommitments,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST SigningCommitments")]
    UnableToDeserializeFrostSigningCommitments,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST SigningPackage")]
    UnableToSerializeFrostSigningPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST SigningPackage")]
    UnableToDeserializeFrostSigningPackage,
    #[cfg(feature = "storage")]
    #[error("Unable to serialize FROST SignatureShare")]
    UnableToSerializeFrostSignature,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST SignatureShare")]
    UnableToDeserializeFrostSignatureShare,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize FROST Signature")]
    UnableToDeserializeFrostSignature,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize bytes into ParticipantMessageData")]
    UnableToDeserializeParticipantMessageData,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize bytes into CoordinatorDataNotFound")]
    UnableToDeserializeCoordinatorDataNotFound,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize bytes into SignedMessagesDataNotFound")]
    UnableToDeserializeSignedMessagesDataNotFound,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize branding data. It's data is corrupted!")]
    UnableToDeserializeBrandingData,
    #[cfg(feature = "storage")]
    #[error("Unable to deserialize AppState data. It's data is corrupted!")]
    UnableToDeserializeAppStateData,
    #[error("Unable to initialize the global storage static `KRILL_STORAGE`")]
    #[cfg(feature = "storage")]
    GlobalStorageInitializeError,
    #[error("Unable to set the state for static `APP_STATE`")]
    UnableToSetAppState,
    #[error("App state machine not initialized")]
    AppStateMachineNotInitialized,
    #[error("{0}")]
    Transmit(String),
    #[cfg(feature = "storage")]
    #[error("Unable to decode the color scheme")]
    UnableToGetColorScheme,
}

#[cfg(feature = "storage")]
impl From<fjall::Error> for KrillError {
    fn from(error: fjall::Error) -> Self {
        match error {
            fjall::Error::Io(io_error) => Self::Io(io_error.kind()),
            _ => Self::Store(error.to_string()),
        }
    }
}
