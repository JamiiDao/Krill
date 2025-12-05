use std::{collections::BTreeMap, future::Future};

use frost_core::Ciphersuite;

use crate::{FrostDkgState, FrostDkgStorage};

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
