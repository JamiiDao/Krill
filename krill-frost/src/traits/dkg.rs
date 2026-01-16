use std::{collections::BTreeMap, future::Future};

use frost_core::Ciphersuite;
use krill_common::KrillResult;

use crate::{FrostDkgState, FrostKeypairData, FrostStorage};

pub trait FrostDkg {
    type DkgCipherSuite: Ciphersuite;

    fn generate_identifier(
        &self,
        identifier: impl AsRef<[u8]>,
    ) -> KrillResult<frost_core::Identifier<Self::DkgCipherSuite>>
        where   <<<Self::DkgCipherSuite as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: std::convert::From<u128>
   ;

    fn generate_identifier_random
    (
        &self,
    ) -> KrillResult<frost_core::Identifier<Self::DkgCipherSuite>>
        where   <<<Self::DkgCipherSuite as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: std::convert::From<u128>
   ;

    fn storage(&self) -> impl FrostStorage<Self::DkgCipherSuite>;

    fn state(&self) -> impl Future<Output = KrillResult<FrostDkgState>>;

    fn frost_dkg_state_transition(&self) -> impl Future<Output = KrillResult<FrostDkgState>>;

    fn signal_dkg(&self) -> impl Future<Output = KrillResult<()>>;

    fn part1(&self) -> impl Future<Output = KrillResult<FrostPart1Output<Self::DkgCipherSuite>>>;

    fn receive_part1(
        &self,
        identifier: frost_core::Identifier<Self::DkgCipherSuite>,
        package: frost_core::keys::dkg::round1::Package<Self::DkgCipherSuite>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn send_part1(
        &self,
    ) -> impl Future<Output = KrillResult<FrostPart1Output<Self::DkgCipherSuite>>>;

    fn part2(&self) -> impl Future<Output = KrillResult<FrostPart2Output<Self::DkgCipherSuite>>>;

    fn receive_part2(
        &self,
        identifier: frost_core::Identifier<Self::DkgCipherSuite>,
        package: frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>,
    ) -> impl Future<Output = KrillResult<()>>;

    fn send_part2(
        &self,
        identifier: &frost_core::Identifier<Self::DkgCipherSuite>,
    ) -> impl Future<
        Output = KrillResult<Option<frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>>>,
    >;

    fn part3(&self) -> impl Future<Output = KrillResult<FrostKeypairData>>;
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
