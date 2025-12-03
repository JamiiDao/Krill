use core::fmt;
use std::collections::BTreeMap;

use async_dup::Arc;
use async_lock::RwLock;
use frost_core::Ciphersuite;
use frost_ed25519::Ed25519Sha512;
use wincode::{SchemaRead, SchemaWrite};
use zeroize::Zeroize;

use crate::{FrostDkgError, FrostDkgState, FrostDkgStorage};

pub type Ed25519IdentifierBytes = Vec<u8>;
pub type Round1PackageBytes = Vec<u8>;
pub type Round2PackageBytes = BTreeMap<Vec<u8>, Vec<u8>>;

#[derive(Zeroize, SchemaRead, SchemaWrite, Default)]
pub struct Part1SecretBytes(Vec<u8>);

impl fmt::Debug for Part1SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part1SecretBytes<Redacted>")
    }
}

#[derive(Zeroize, SchemaRead, SchemaWrite, Default)]
pub struct Part2SecretBytes(Vec<u8>);

impl fmt::Debug for Part2SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part2SecretBytes<Redacted>")
    }
}

#[derive(Debug, SchemaRead, SchemaWrite, Default)]
pub struct FrostDkgMemStorage {
    identifier: Vec<u8>,
    context_string: &'static str,
    maximum_signers: u16,
    minimum_signers: u16,
    dkg_state: FrostDkgState,
    part1_secret: Part1SecretBytes,
    part1_package: Round1PackageBytes,
    received_part1_packages: BTreeMap<Ed25519IdentifierBytes, Round1PackageBytes>,
    part2_secret: Part2SecretBytes,
    part2_package: Round2PackageBytes,
    received_part2_packages: BTreeMap<Ed25519IdentifierBytes, Round1PackageBytes>,
}

impl FrostDkgMemStorage {
    pub fn init() -> Self {
        Self::default()
    }
}

impl FrostDkgEd25519Storage for Arc<RwLock<FrostDkgMemStorage>> {}

impl<C: Ciphersuite, E: core::error::Error + std::convert::From<FrostDkgError>>
    FrostDkgStorage<C, E> for Arc<RwLock<FrostDkgMemStorage>>
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
        self.write().await.identifier = identifier.serialize();

        Ok(())
    }

    async fn get_identifier(&self) -> Result<frost_core::Identifier<C>, E> {
        Ok(
            frost_core::Identifier::<C>::deserialize(&self.0.read().await.identifier).map_err(
                |error| FrostDkgError::Ed25519Sha512IdentifierDeserialize(error.to_string()),
            )?,
        )
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
        mut secret: frost_core::keys::dkg::round1::SecretPackage<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> Result<(), E> {
        let secret_bytes = secret
            .serialize()
            .map_err(|error| FrostDkgError::Ed25519Sha512Part1SecretPackage(error.to_string()))?;
        let package_bytes = package
            .serialize()
            .map_err(|error| FrostDkgError::Ed25519Sha512Part1Package(error.to_string()))?;

        self.write().await.part1_secret = Part1SecretBytes(secret_bytes);
        self.write().await.part1_package = package_bytes;

        secret.zeroize();

        Ok(())
    }

    /// This zeroizes the secret so its only accessible once
    async fn get_part1_secret_package(
        &self,
    ) -> Result<frost_core::keys::dkg::round1::SecretPackage<C>, E> {
        let secret = frost_core::keys::dkg::round1::SecretPackage::<C>::deserialize(
            &self.read().await.part1_secret.0,
        )
        .map_err(|error| FrostDkgError::Ed25519Sha512Part1SecretDeserialize(error.to_string()))?;

        self.write().await.part1_secret.zeroize();

        Ok(secret)
    }

    async fn get_part1_public_package(
        &self,
    ) -> Result<frost_core::keys::dkg::round1::Package<C>, E> {
        Ok(frost_core::keys::dkg::round1::Package::<C>::deserialize(
            &self.read().await.part1_package,
        )
        .map_err(|error| {
            FrostDkgError::Ed25519Sha512Part1PublicPackageDeserialize(error.to_string())
        })?)
    }

    async fn add_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> Result<(), E> {
        let identifier_bytes = identifier.serialize();
        let package_bytes = package.serialize().map_err(|error| {
            FrostDkgError::Ed25519SerializeReceivedPart1Package(error.to_string())
        })?;

        self.write()
            .await
            .received_part1_packages
            .insert(identifier_bytes, package_bytes);

        Ok(())
    }

    async fn get_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> Result<Option<frost_core::keys::dkg::round1::Package<C>>, E> {
        let identifier_bytes = identifier.serialize();

        Ok(self
            .read()
            .await
            .received_part1_packages
            .get(&identifier_bytes)
            .map(|value| {
                frost_core::keys::dkg::round1::Package::deserialize(value).map_err(|error| {
                    FrostDkgError::Ed25519DeserializeReceivedPart1Package(error.to_string())
                })
            })
            .transpose()?)
    }

    async fn has_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> Result<bool, E> {
        let identifier_bytes = identifier.serialize();

        Ok(self
            .read()
            .await
            .received_part1_packages
            .contains_key(&identifier_bytes))
    }

    async fn get_all_part1_received_packages(
        &self,
    ) -> Result<BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>, E>
    {
        let mut packages = BTreeMap::<
            frost_core::Identifier<C>,
            frost_core::keys::dkg::round1::Package<C>,
        >::default();
        self.read()
            .await
            .received_part1_packages
            .iter()
            .try_for_each(|(key, value)| {
                let identifier = frost_core::Identifier::deserialize(key).map_err(|error| {
                    FrostDkgError::Ed25519Sha512IdentifierDeserializeAll(error.to_string())
                })?;
                let package = frost_core::keys::dkg::round1::Package::deserialize(value).map_err(
                    |error| FrostDkgError::Ed25519Part1DeserializeAll(error.to_string()),
                )?;

                packages.insert(identifier, package);

                Ok(())
            })?;

        Ok(packages)
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
        packages: &BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    ) -> Result<(), E> {
        let secret_bytes = Part2SecretBytes(secret.serialize().map_err(|error| {
            FrostDkgError::Ed25519Sha512Part2SecretSerialize(error.to_string())
        })?);
        let package_bytes = packages
            .iter()
            .map(|(identifier, package)| {
                let identifier_bytes = identifier.serialize();
                let part2_package_bytes = package.serialize().map_err(|error| {
                    FrostDkgError::Ed25519Sha512Part2PackageDeserialize(error.to_string())
                })?;

                Ok((identifier_bytes, part2_package_bytes))
            })
            .collect::<Result<Round2PackageBytes, E>>()?;

        self.write().await.part2_secret = secret_bytes;
        self.write().await.part2_package = package_bytes;

        Ok(())
    }

    async fn get_part2_secret(&self) -> Result<frost_core::keys::dkg::round2::SecretPackage<C>, E> {
        let secret = frost_core::keys::dkg::round2::SecretPackage::deserialize(
            self.write().await.part2_secret.0.as_slice(),
        )
        .map_err(|error| FrostDkgError::Ed25519Sha512Part2SecretDeserialize(error.to_string()))?;

        self.write().await.part2_secret.zeroize();

        Ok(secret)
    }

    async fn get_part2_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> Result<Option<frost_core::keys::dkg::round2::Package<C>>, E> {
        Ok(self
            .write()
            .await
            .part2_package
            .get(&identifier.serialize())
            .map(|package_bytes| {
                frost_core::keys::dkg::round2::Package::<C>::deserialize(package_bytes).map_err(
                    |error| FrostDkgError::Ed25519DeserializePart2Package(error.to_string()),
                )
            })
            .transpose()?)
    }

    async fn get_all_part2_received_packages(
        &self,
    ) -> Result<BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>, E>
    {
        self.write()
            .await
            .received_part2_packages
            .iter()
            .map(|(identifier_bytes, package_bytes)| {
                let identifier =
                    frost_core::Identifier::deserialize(identifier_bytes).map_err(|error| {
                        FrostDkgError::Ed25519Sha512IdentifierDeserializePart2(error.to_string())
                    })?;
                let package = frost_core::keys::dkg::round2::Package::deserialize(package_bytes)
                    .map_err(|error| {
                        FrostDkgError::Ed25519PackageDeserializePart2(error.to_string())
                    })?;

                Ok((identifier, package))
            })
            .collect::<Result<
                BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
                E,
            >>()
    }

    async fn add_part2_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round2::Package<C>,
    ) -> Result<(), E> {
        let package_bytes = package.serialize().map_err(|error| {
            FrostDkgError::Ed25519Sha512Round2PackageSerialize(error.to_string())
        })?;

        self.write()
            .await
            .received_part2_packages
            .insert(identifier.serialize(), package_bytes);

        Ok(())
    }

    async fn clear(&self) -> Result<(), E> {
        let mut take = self.write().await;
        take.part1_package.clear();
        take.part2_package.clear();
        take.received_part1_packages.clear();
        take.received_part2_packages.clear();
        take.identifier.clear();
        take.dkg_state = FrostDkgState::Initial;
        take.maximum_signers = 0;
        take.minimum_signers = 0;

        Ok(())
    }
}

pub trait FrostDkgEd25519Storage: FrostDkgStorage<Ed25519Sha512, FrostDkgError> {}
