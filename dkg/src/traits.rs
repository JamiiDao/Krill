use std::{collections::BTreeMap, future::Future};

use frost_core::Ciphersuite;
use frost_ed25519::Ed25519Sha512;

use crate::{FrostDkgError, FrostDkgState};

pub trait FrostDkg {
    type DkgGenericError: core::error::Error;
    type DkgCipherSuite: Ciphersuite;

    fn storage(
        &self,
    ) -> impl Future<
        Output = Result<
            impl FrostDkgStorage<Self::DkgCipherSuite, Self::DkgGenericError>,
            Self::DkgGenericError,
        >,
    >;

    fn state(&self) -> impl Future<Output = Result<FrostDkgState, Self::DkgGenericError>>;

    fn generate_identifier(
        &self,
        identifier: impl AsRef<[u8]>,
    ) -> Result<frost_core::Identifier<Self::DkgCipherSuite>, Self::DkgGenericError>;

    fn generate_identifier_random(
        &self,
    ) -> Result<frost_core::Identifier<Self::DkgCipherSuite>, Self::DkgGenericError>;

    fn frost_dkg_state_transition(
        &self,
    ) -> impl Future<Output = Result<FrostDkgState, Self::DkgGenericError>>;

    fn part1(
        &self,
    ) -> impl Future<Output = Result<FrostPart1Output<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn receive_part1(
        &self,
        identifier: frost_core::Identifier<Self::DkgCipherSuite>,
        package: frost_core::keys::dkg::round1::Package<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<(), Self::DkgGenericError>>;

    fn send_part1(
        &self,
    ) -> impl Future<Output = Result<FrostPart1Output<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn part2(
        &self,
    ) -> impl Future<Output = Result<FrostPart2Output<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn receive_part2(
        &self,
        identifier: frost_core::Identifier<Self::DkgCipherSuite>,
        package: frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>,
    ) -> impl Future<Output = Result<(), Self::DkgGenericError>>;

    fn send_part2(
        &self,
        identifier: &frost_core::Identifier<Self::DkgCipherSuite>,
    ) -> impl Future<
        Output = Result<
            Option<frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>>,
            Self::DkgGenericError,
        >,
    >;

    fn part3(
        &self,
    ) -> impl Future<Output = Result<FrostPart3Output<Self::DkgCipherSuite>, Self::DkgGenericError>>;
}

pub trait FrostDkgEd25519Storage: FrostDkgStorage<Ed25519Sha512, FrostDkgError> {}

pub trait FrostDkgStorage<C: Ciphersuite, E: core::error::Error> {
    fn set_context_string(&self, context_str: &'static str) -> impl Future<Output = Result<(), E>>;

    fn get_context_string(&self) -> impl Future<Output = Result<&'static str, E>>;

    fn set_state(&self, state: FrostDkgState) -> impl Future<Output = Result<(), E>>;

    fn get_state(&self) -> impl Future<Output = Result<FrostDkgState, E>>;

    fn set_identifier(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> impl Future<Output = Result<(), E>>;

    fn get_identifier(&self) -> impl Future<Output = Result<frost_core::Identifier<C>, E>>;

    fn set_maximum_signers(&self, maximum_signers: u16) -> impl Future<Output = Result<(), E>>;

    fn get_maximum_signers(&self) -> impl Future<Output = Result<u16, E>>;

    fn set_minimum_signers(&self, minimum_signers: u16) -> impl Future<Output = Result<(), E>>;

    fn get_minimum_signers(&self) -> impl Future<Output = Result<u16, E>>;

    fn set_part1_package(
        &self,
        secret: frost_core::keys::dkg::round1::SecretPackage<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> impl Future<Output = Result<(), E>>;

    fn get_part1_secret_package(
        &self,
    ) -> impl Future<Output = Result<frost_core::keys::dkg::round1::SecretPackage<C>, E>>;

    fn get_part1_public_package(
        &self,
    ) -> impl Future<Output = Result<frost_core::keys::dkg::round1::Package<C>, E>>;

    fn add_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> impl Future<Output = Result<(), E>>;

    fn has_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> impl Future<Output = Result<bool, E>>;

    fn get_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> impl Future<Output = Result<Option<frost_core::keys::dkg::round1::Package<C>>, E>>;

    fn get_all_part1_received_packages(
        &self,
    ) -> impl Future<
        Output = Result<
            BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>,
            E,
        >,
    >;

    fn part1_received_packages_count(&self) -> impl Future<Output = Result<usize, E>>;

    fn part2_received_packages_count(&self) -> impl Future<Output = Result<usize, E>>;

    fn set_part2_package(
        &self,
        secret: frost_core::keys::dkg::round2::SecretPackage<C>,
        packages: &BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    ) -> impl Future<Output = Result<(), E>>;

    fn add_part2_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round2::Package<C>,
    ) -> impl Future<Output = Result<(), E>>;

    fn get_part2_secret(
        &self,
    ) -> impl Future<Output = Result<frost_core::keys::dkg::round2::SecretPackage<C>, E>>;

    fn get_part2_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> impl Future<Output = Result<Option<frost_core::keys::dkg::round2::Package<C>>, E>>;

    fn get_all_part2_received_packages(
        &self,
    ) -> impl Future<
        Output = Result<
            BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
            E,
        >,
    >;

    fn clear(&self) -> impl Future<Output = Result<(), E>>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FrostPart1Output<C: Ciphersuite> {
    pub identifier: frost_core::Identifier<C>,
    pub package: frost_core::keys::dkg::round1::Package<C>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FrostPart2Output<C: Ciphersuite> {
    pub identifier: frost_core::Identifier<C>,
    pub packages: BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FrostPart3Output<C: Ciphersuite> {
    pub identifier: frost_core::Identifier<C>,
    pub maximum_signers: u16,
    pub minimum_signers: u16,
    pub secret: frost_core::keys::KeyPackage<C>,
    pub public_package: frost_core::keys::PublicKeyPackage<C>,
}

impl<C: Ciphersuite> FrostPart3Output<C> {
    pub fn identifier(&self) -> &frost_core::Identifier<C> {
        &self.identifier
    }

    pub fn maximum_signers(&self) -> u16 {
        self.maximum_signers
    }

    pub fn minimum_signers(&self) -> u16 {
        self.minimum_signers
    }

    pub fn secret(&self) -> &frost_core::keys::KeyPackage<C> {
        &self.secret
    }

    pub fn public_package(&self) -> &frost_core::keys::PublicKeyPackage<C> {
        &self.public_package
    }
}
