pub type FrostDkgResult<T> = Result<T, FrostDkgError>;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum FrostDkgError {
    #[error("Global storage is not initialized yet it is being called")]
    GlobalStorageNotInitialized,
    #[error("The maximum number of signers must be equal to or greater than the minimum number of signers")]
    MinimumSignersGreaterThanMaximumSigners,
    #[error("Unable to derive a DKG Identifier from random bytes")]
    IdentifierDerivationNotSupported,
    #[error("Frost Identifier already exists in storage")]
    IdentifierAlreadyExists,
    #[error("There must be at least two signers for perform distributed key generation")]
    ThereMustBeAtLeast2Signers,
    #[error("Unable to serialize an Ed25519 part1 secret package. Error: `{0}`.")]
    Ed25519Sha512Part1SecretPackage(String),
    #[error("Unable to serialize an Ed25519 part1 public package. Error: `{0}`.")]
    Ed25519Sha512Part1Package(String),
    #[error("Unable to deserialize an Ed25519Sha512 Identifier. Error: `{0}`.")]
    Ed25519Sha512IdentifierDeserialize(String),
    #[error("Unable to deserialize an Ed25519Sha512 Part1 secret. Error: `{0}`.")]
    Ed25519Sha512Part1SecretDeserialize(String),
    #[error("Unable to deserialize an Ed25519Sha512 Part1 public package. Error: `{0}`.")]
    Ed25519Sha512Part1PublicPackageDeserialize(String),
    #[error("Attempted to transition FROST DKG state yet the state is already finalized")]
    DkgStateAlreadyFinalized,
    #[error("Invalid FROST DKG state. Error: `{0}`.")]
    InvalidDkgState(&'static str),
    #[error("Unable to perform key generation for part 1. Error: `{0}`")]
    Part1KeyGenerationError(String),
    #[error("Unable to serialize the received part1 package. Error: `{0}`.")]
    Ed25519SerializeReceivedPart1Package(String),
    #[error(
        "Unable to deserialize the received part1 package fetched from storage. Error: `{0}`."
    )]
    Ed25519DeserializeReceivedPart1Package(String),
    #[error(
        "Unable to deserialize the identifier when fetching all part1 packages. Error: `{0}`."
    )]
    Ed25519Sha512IdentifierDeserializeAll(String),
    #[error(
        "Unable to deserialize the part1 package when fetching all part1 packages. Error: `{0}`."
    )]
    Ed25519Part1DeserializeAll(String),
    #[error("The maximum number of parties has been reached yet more part1 packages have been received.")]
    Part1MaximumPartiesReached,
    #[error("The maximum number of parties has been reached yet more part2 packages have been received.")]
    Part2MaximumPartiesReached,
    #[error("Unable to serialize part2 secret into bytes for storage")]
    Ed25519Sha512Part2SecretSerialize(String),
    #[error("Unable to deserialize part2 secret into bytes for storage")]
    Ed25519Sha512Part2SecretDeserialize(String),
    #[error("Unable to serialize part2 public package into bytes for storage")]
    Ed25519Sha512Part2PackageDeserialize(String),
    #[error("Current FROST DKG state is `{0}` yet FROST DKG state is supposed to be Part 2")]
    InvalidFrostDkgState(String),
    #[error("Unable to perform key generation for part 2. Error: `{0}`.")]
    Part2KeyGenerationError(String),
    #[error("Unable to deserialize FROST DKG part2 package. Error: `{0}`.")]
    Ed25519DeserializePart2Package(String),
    #[error("Unable to deserialize FROST DKG part 2 identifier from storage. Error: `{0}`.")]
    Ed25519Sha512IdentifierDeserializePart2(String),
    #[error("Unable to deserialize FROST DKG part 2 package from storage. Error: `{0}`.")]
    Ed25519PackageDeserializePart2(String),
    #[error("Unable to serialize a FROST DKG part 2 received package for storage. Error: `{0}`.")]
    Ed25519Sha512Round2PackageSerialize(String),
    #[error("Unable to finalize FROST DKG part3. Error: `{0}`.")]
    Part3Finalize(String),
    #[error("Unable to create a fixed size byte array from a slice. Err: `{0}`.")]
    ToByteArray(&'static str),
}
