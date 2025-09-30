use std::{collections::BTreeMap, future::Future};

use frost_core::{Ciphersuite, Identifier};

use crate::DkgState;

pub trait FrostDkg {
    type DkgGenericError: core::error::Error;
    type DkgCipherSuite: Ciphersuite;

    fn storage(
        &self,
    ) -> impl Future<Output = Result<impl FrostDkgStorage<Self::DkgCipherSuite>, Self::DkgGenericError>>;

    fn generate_identifier(
        &self,
    ) -> Result<frost_core::Identifier<Self::DkgCipherSuite>, Self::DkgGenericError>;

    // fn set_part1_packages(
    //     &self,
    //     secret: frost_core::keys::dkg::round1::SecretPackage<Self::DkgCipherSuite>,
    //     package: frost_core::keys::dkg::round1::Package<Self::DkgCipherSuite>,
    // ) -> impl Future<Output = Result<(), Self::DkgGenericError>>;

    // /// Sets packages for other parties received via a trusted communication channel
    // fn set_part1_received_package(
    //     &self,
    //     identifier: Identifier<Self::DkgCipherSuite>,
    //     package: frost_core::keys::dkg::round1::Package<Self::DkgCipherSuite>,
    // ) -> impl Future<Output = Result<(), Self::DkgGenericError>>;
}

pub trait FrostDkgStorage<C: Ciphersuite> {
    fn set_context_string(
        &self,
        context_str: &'static str,
    ) -> impl Future<Output = Result<(), impl core::error::Error>>;

    fn get_context_string(
        &self,
    ) -> impl Future<Output = Result<&'static str, impl core::error::Error>>;

    fn set_state(
        &self,
        state: DkgState,
    ) -> impl Future<Output = Result<(), impl core::error::Error>>;

    fn get_state(&self) -> impl Future<Output = Result<DkgState, impl core::error::Error>>;

    fn set_identifier(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> impl Future<Output = Result<(), impl core::error::Error>>;

    fn get_identifier(
        &self,
    ) -> impl Future<Output = Result<frost_core::Identifier<C>, impl core::error::Error>>;

    fn set_maximum_signers(
        &self,
        maximum_signers: u16,
    ) -> impl Future<Output = Result<(), impl core::error::Error>>;

    fn get_maximum_signers(&self) -> impl Future<Output = Result<u16, impl core::error::Error>>;

    fn set_minimum_signers(
        &self,
        minimum_signers: u16,
    ) -> impl Future<Output = Result<(), impl core::error::Error>>;

    fn get_minimum_signers(&self) -> impl Future<Output = Result<u16, impl core::error::Error>>;
}
