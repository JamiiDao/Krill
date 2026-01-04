use crate::{FrostDkgMemStorage, FrostGenericDkg, FrostGenericSigning, FrostSigningMemStorage};

pub type FrostEd25519DkgMemStorage = FrostDkgMemStorage<frost_ed25519::Ed25519Sha512>;

pub type FrostEd25519Dkg<S> = FrostGenericDkg<frost_ed25519::Ed25519Sha512, S>;

pub type FrostEd25519SigningStorage = FrostSigningMemStorage<frost_ed25519::Ed25519Sha512>;

pub type FrostEd25519Signing<S> = FrostGenericSigning<frost_ed25519::Ed25519Sha512, S>;
