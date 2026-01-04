use std::collections::BTreeMap;

use async_dup::Arc;
use async_lock::RwLock;
use frost_core::{
    keys::dkg::{
        round1::{Package as Round1Package, SecretPackage as Round1SecretPackage},
        round2::{Package as Round2Package, SecretPackage as Round2SecretPackage},
    },
    Ciphersuite,
};
use serde::{Deserialize, Serialize};

use crate::{FrostDkgError, FrostDkgState, FrostDkgStorage};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FrostDkgMemStorage<C: Ciphersuite> {
    identifier: Option<frost_core::Identifier<C>>,
    context_string: &'static str,
    maximum_signers: u16,
    minimum_signers: u16,
    dkg_state: FrostDkgState,
    part1_secret: Option<Round1SecretPackage<C>>,
    part1_package: Option<Round1Package<C>>,
    received_part1_packages: BTreeMap<frost_core::Identifier<C>, Round1Package<C>>,
    part2_secret: Option<Round2SecretPackage<C>>,
    part2_package: BTreeMap<frost_core::Identifier<C>, Round2Package<C>>,
    received_part2_packages: BTreeMap<frost_core::Identifier<C>, Round2Package<C>>,
}

impl<C: Ciphersuite> FrostDkgMemStorage<C> {
    pub fn init() -> Self {
        Self {
            identifier: Option::default(),
            part1_secret: Option::default(),
            part1_package: Option::default(),
            received_part1_packages: BTreeMap::default(),
            part2_secret: Option::default(),
            part2_package: BTreeMap::default(),
            received_part2_packages: BTreeMap::default(),
            context_string: "",
            maximum_signers: 2u16,
            minimum_signers: 2u16,
            dkg_state: FrostDkgState::default(),
        }
    }
}

impl<C: Ciphersuite, E: core::error::Error + std::convert::From<FrostDkgError>>
    FrostDkgStorage<C, E> for Arc<RwLock<FrostDkgMemStorage<C>>>
{
    async fn set_context_string(&self, context_string: &'static str) -> Result<(), E> {
        self.write().await.context_string = context_string;

        Ok(())
    }

    async fn get_context_string(&self) -> Result<&'static str, E> {
        Ok(self.read().await.context_string)
    }

    async fn set_state(&self, dkg_state: FrostDkgState) -> Result<(), E> {
        self.write().await.dkg_state = dkg_state;

        Ok(())
    }

    async fn get_state(&self) -> Result<FrostDkgState, E> {
        Ok(self.read().await.dkg_state)
    }

    async fn set_identifier(&self, identifier: frost_core::Identifier<C>) -> Result<(), E> {
        self.write().await.identifier.replace(identifier);

        Ok(())
    }

    async fn get_identifier(&self) -> Result<frost_core::Identifier<C>, E> {
        Ok(self
            .0
            .read()
            .await
            .identifier
            .ok_or(FrostDkgError::DkgIdentifierNotFound)?)
    }

    async fn set_maximum_signers(&self, maximum_signers: u16) -> Result<(), E> {
        self.write().await.maximum_signers = maximum_signers;

        Ok(())
    }

    async fn get_maximum_signers(&self) -> Result<u16, E> {
        Ok(self.read().await.maximum_signers)
    }

    async fn set_minimum_signers(&self, minimum_signers: u16) -> Result<(), E> {
        self.write().await.minimum_signers = minimum_signers;

        Ok(())
    }

    async fn get_minimum_signers(&self) -> Result<u16, E> {
        Ok(self.read().await.minimum_signers)
    }

    async fn set_part1_package(
        &self,
        secret: frost_core::keys::dkg::round1::SecretPackage<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> Result<(), E> {
        self.write().await.part1_secret.replace(secret);
        self.write().await.part1_package.replace(package);

        Ok(())
    }

    /// This zeroizes the secret so its only accessible once
    async fn get_part1_secret_package(
        &self,
    ) -> Result<frost_core::keys::dkg::round1::SecretPackage<C>, E> {
        Ok(self
            .write()
            .await
            .part1_secret
            .take()
            .ok_or(FrostDkgError::Round1SecretNotFound)?)
    }

    async fn get_part1_public_package(
        &self,
    ) -> Result<frost_core::keys::dkg::round1::Package<C>, E> {
        Ok(self
            .read()
            .await
            .part1_package
            .clone()
            .ok_or(FrostDkgError::Part1PublicPackageNotFound)?)
    }

    async fn add_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> Result<(), E> {
        self.write()
            .await
            .received_part1_packages
            .insert(identifier, package);

        Ok(())
    }

    async fn get_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> Result<Option<frost_core::keys::dkg::round1::Package<C>>, E> {
        Ok(self
            .read()
            .await
            .received_part1_packages
            .get(&identifier)
            .cloned())
    }

    async fn has_part1_received_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> Result<bool, E> {
        Ok(self
            .read()
            .await
            .received_part1_packages
            .contains_key(identifier))
    }

    async fn get_all_part1_received_packages(
        &self,
    ) -> Result<BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>, E>
    {
        Ok(self.read().await.received_part1_packages.clone())
    }

    async fn part1_received_packages_count(&self) -> Result<usize, E> {
        Ok(self.read().await.received_part1_packages.len())
    }

    async fn part2_received_packages_count(&self) -> Result<usize, E> {
        Ok(self.read().await.received_part2_packages.len())
    }

    async fn set_part2_package(
        &self,
        secret: frost_core::keys::dkg::round2::SecretPackage<C>,
        packages: BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    ) -> Result<(), E> {
        self.write().await.part2_secret.replace(secret);
        self.write().await.part2_package = packages;

        Ok(())
    }

    async fn get_part2_secret(&self) -> Result<frost_core::keys::dkg::round2::SecretPackage<C>, E> {
        Ok(self
            .write()
            .await
            .part2_secret
            .take()
            .ok_or(FrostDkgError::Part2SecretNotFound)?)
    }

    async fn get_part2_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> Result<Option<frost_core::keys::dkg::round2::Package<C>>, E> {
        Ok(self.write().await.part2_package.get(identifier).cloned())
    }

    async fn get_all_part2_received_packages(
        &self,
    ) -> Result<BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>, E>
    {
        Ok(self.write().await.received_part2_packages.clone())
    }

    async fn add_part2_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round2::Package<C>,
    ) -> Result<(), E> {
        self.write()
            .await
            .received_part2_packages
            .insert(identifier, package);

        Ok(())
    }

    async fn clear(&self) -> Result<(), E> {
        let mut take = self.write().await;
        take.part1_package.take();
        take.part2_package.clear();
        take.received_part1_packages.clear();
        take.received_part2_packages.clear();
        take.identifier.take();
        take.dkg_state = FrostDkgState::Initial;
        take.maximum_signers = 0;
        take.minimum_signers = 0;

        Ok(())
    }
}
