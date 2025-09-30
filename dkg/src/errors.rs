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
    #[error("Unable to deserialize an Ed25519Sha512 Identifier. Error: `{0}`.")]
    Ed25519Sha512IdentifierDeserialize(String),
}
