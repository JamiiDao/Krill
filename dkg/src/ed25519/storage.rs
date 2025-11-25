use std::collections::{BTreeMap, HashMap};

use async_dup::Arc;
use async_lock::RwLock;
use frost_core::Ciphersuite;
use frost_ed25519::{
    keys::{
        dkg::{
            round1::{
                Package as Ed25519Round1Package, SecretPackage as Ed25519Round1SecretPackage,
            },
            round2::{
                Package as Ed25519Round2Package, SecretPackage as Ed25519Round2SecretPackage,
            },
        },
        KeyPackage as Ed25519KeyPackage, PublicKeyPackage as Ed25519PublicKeyPackage,
    },
    Error, Identifier as Ed25519Identifier,
};
use wincode::{SchemaRead, SchemaWrite};

use crate::{DkgState, FrostDkgError, FrostDkgResult, FrostDkgStorage, FROST_ED25519_MEM_STORAGE};

#[derive(Debug, PartialEq, SchemaRead, SchemaWrite, Default)]
pub struct FrostDkgMemStorage {
    identifier: Vec<u8>,
    context_string: &'static str,
    maximum_signers: u16,
    minimum_signers: u16,
    dkg_state: DkgState,
}

impl FrostDkgMemStorage {
    pub fn init() -> Self {
        Self::default()
    }

    pub async fn get_storage() -> FrostDkgResult<Arc<RwLock<Self>>> {
        FROST_ED25519_MEM_STORAGE
            .get()
            .cloned()
            .ok_or(FrostDkgError::GlobalStorageNotInitialized)
    }
}

impl<C: Ciphersuite> FrostDkgStorage<C> for Arc<RwLock<FrostDkgMemStorage>> {
    async fn set_context_string(
        &self,
        context_string: &'static str,
    ) -> Result<(), impl core::error::Error> {
        self.write().await.context_string = context_string;

        Ok::<_, FrostDkgError>(())
    }

    async fn get_context_string(&self) -> Result<&'static str, impl core::error::Error> {
        Ok::<_, FrostDkgError>(self.read().await.context_string)
    }

    async fn set_state(&self, dkg_state: DkgState) -> Result<(), impl core::error::Error> {
        self.write().await.dkg_state = dkg_state;

        Ok::<(), FrostDkgError>(())
    }

    async fn get_state(&self) -> Result<DkgState, impl core::error::Error> {
        Ok::<_, FrostDkgError>(self.read().await.dkg_state)
    }

    async fn set_identifier(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> Result<(), impl core::error::Error> {
        self.write().await.identifier = identifier.serialize();

        Ok::<_, FrostDkgError>(())
    }

    async fn get_identifier(&self) -> Result<frost_core::Identifier<C>, impl core::error::Error> {
        frost_core::Identifier::<C>::deserialize(&self.0.read().await.identifier)
            .map_err(|error| FrostDkgError::Ed25519Sha512IdentifierDeserialize(error.to_string()))
    }

    async fn set_maximum_signers(
        &self,
        maximum_signers: u16,
    ) -> Result<(), impl core::error::Error> {
        self.write().await.maximum_signers = maximum_signers;

        Ok::<_, FrostDkgError>(())
    }

    async fn get_maximum_signers(&self) -> Result<u16, impl core::error::Error> {
        Ok::<_, FrostDkgError>(self.read().await.maximum_signers)
    }

    async fn set_minimum_signers(
        &self,
        minimum_signers: u16,
    ) -> Result<(), impl core::error::Error> {
        self.write().await.minimum_signers = minimum_signers;

        Ok::<_, FrostDkgError>(())
    }

    async fn get_minimum_signers(&self) -> Result<u16, impl core::error::Error> {
        Ok::<_, FrostDkgError>(self.read().await.minimum_signers)
    }
}
