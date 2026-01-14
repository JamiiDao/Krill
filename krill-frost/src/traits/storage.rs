use std::{collections::BTreeMap, future::Future};

use frost_core::Ciphersuite;

use crate::{FrostDkgData, FrostDkgState, KrillResult};

pub trait FrostDkgStorage<C: Ciphersuite> {
    fn serialize(&self, data: &FrostDkgData) -> Vec<u8>;

    fn deserialize(&self, bytes: &[u8]) -> KrillResult<FrostDkgData>;

    fn set_state(&self, state: FrostDkgState) -> impl Future<Output = KrillResult<()>>;

    fn get_state(&self) -> impl Future<Output = KrillResult<FrostDkgState>>;

    fn set_identifier(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn get_identifier(&self) -> impl Future<Output = KrillResult<frost_core::Identifier<C>>>;

    fn set_maximum_signers(&self, maximum_signers: u16) -> impl Future<Output = KrillResult<()>>;

    fn get_maximum_signers(&self) -> impl Future<Output = KrillResult<u16>>;

    fn set_minimum_signers(&self, minimum_signers: u16) -> impl Future<Output = KrillResult<()>>;

    fn get_minimum_signers(&self) -> impl Future<Output = KrillResult<u16>>;

    fn set_part1_package(
        &self,
        secret: frost_core::keys::dkg::round1::SecretPackage<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn get_part1_secret_package(
        &self,
    ) -> impl Future<Output = KrillResult<frost_core::keys::dkg::round1::SecretPackage<C>>>;

    fn get_part1_public_package(
        &self,
    ) -> impl Future<Output = KrillResult<frost_core::keys::dkg::round1::Package<C>>>;

    fn add_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round1::Package<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn has_part1_received_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<bool>>;

    fn get_part1_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<Option<frost_core::keys::dkg::round1::Package<C>>>>;

    fn get_all_part1_received_packages(
        &self,
    ) -> impl Future<
        Output = KrillResult<
            BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>,
        >,
    >;

    fn part1_received_packages_count(&self) -> impl Future<Output = KrillResult<usize>>;

    fn part2_received_packages_count(&self) -> impl Future<Output = KrillResult<usize>>;

    fn set_part2_package(
        &self,
        secret: frost_core::keys::dkg::round2::SecretPackage<C>,
        packages: BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn add_part2_received_package(
        &self,
        identifier: frost_core::Identifier<C>,
        package: frost_core::keys::dkg::round2::Package<C>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn get_part2_secret(
        &self,
    ) -> impl Future<Output = KrillResult<frost_core::keys::dkg::round2::SecretPackage<C>>>;

    fn get_part2_package(
        &self,
        identifier: &frost_core::Identifier<C>,
    ) -> impl Future<Output = KrillResult<Option<frost_core::keys::dkg::round2::Package<C>>>>;

    fn get_all_part2_received_packages(
        &self,
    ) -> impl Future<
        Output = KrillResult<
            BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
        >,
    >;

    fn clear_dkg_data(&self) -> impl Future<Output = KrillResult<()>>;
}
