use async_dup::Arc;
use async_lock::RwLock;
use frost_core::Ciphersuite;
use frost_ed25519::{
    self as frost,
    keys::dkg::{
        round1::{Package as Ed25519Round1Package, SecretPackage as Ed25519Round1SecretPackage},
        round2::{Package as Ed25519Round2Package, SecretPackage as Ed25519Round2SecretPackage},
    },
    Ed25519Sha512, Identifier as Ed25519Identifier,
};

use crate::{
    DkgState, FrostDkg, FrostDkgError, FrostDkgMemStorage, FrostDkgResult, FrostDkgStorage,
    RandomBytes,
};

#[derive(Default)]
pub struct FrostEd25519Dkg;

impl FrostEd25519Dkg {
    pub fn new() -> Self {
        Self
    }
}

impl FrostDkg for FrostEd25519Dkg {
    type DkgCipherSuite = Ed25519Sha512;
    type DkgGenericError = FrostDkgError;

    async fn storage(
        &self,
    ) -> Result<impl crate::FrostDkgStorage<Self::DkgCipherSuite>, Self::DkgGenericError> {
        Ok(FrostDkgMemStorage::get_storage().await?.clone())
    }

    fn generate_identifier(&self) -> Result<Ed25519Identifier, FrostDkgError> {
        let identifier = RandomBytes::<8>::generate();
        let identifier = u64::from_le_bytes(*identifier.take());
        Ed25519Identifier::new(identifier.into())
            .or(Err(FrostDkgError::IdentifierDerivationNotSupported))
    }
}
