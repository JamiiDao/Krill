use std::marker::PhantomData;

use frost_core::Ciphersuite;
use zeroize::Zeroize;

use crate::{
    FrostDkg, FrostDkgState, FrostIdentifier, FrostKeypairData, FrostPart1Output, FrostPart2Output,
    FrostSigningKeyPackage, FrostSigningPublicKeyPackage, FrostStorage, IdentifierGenerator,
    KrillError, KrillResult,
};

pub struct FrostGenericDkg<C: Ciphersuite, S: FrostStorage<C>>(S, PhantomData<C>);

impl<C: Ciphersuite, S: FrostStorage<C>> FrostGenericDkg<C, S> {
    pub fn new(storage: S) -> Self {
        Self(storage, PhantomData)
    }
}

impl<C: Ciphersuite, S: FrostStorage<C> + Clone> FrostDkg for FrostGenericDkg<C, S> {
    type DkgCipherSuite = C;

    fn storage(&self) -> impl FrostStorage<Self::DkgCipherSuite> {
        self.0.clone()
    }

    fn generate_identifier(
        &self,
        identifier: impl AsRef<[u8]>,
    ) -> KrillResult<frost_core::Identifier<C>>
        where   <<<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: std::convert::From<u128>
    {
        IdentifierGenerator::hashed_identifier(identifier.as_ref())
    }

    fn generate_identifier_random
    (
        &self,
    ) -> KrillResult<frost_core::Identifier<C>>
        where   <<<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: std::convert::From<u128>
    {
        IdentifierGenerator::random_identifier()
    }

    async fn signal_dkg(&self) -> KrillResult<()> {
        self.storage().clear_dkg_data().await
    }

    async fn state(&self) -> KrillResult<FrostDkgState> {
        self.storage().get_state().await
    }

    async fn frost_dkg_state_transition(&self) -> KrillResult<FrostDkgState> {
        let current_state = self.storage().get_state().await?;

        let state = match current_state {
            FrostDkgState::Initial => FrostDkgState::Part1,
            FrostDkgState::Part1 => FrostDkgState::Part2,
            FrostDkgState::Part2 => FrostDkgState::Part3,
            FrostDkgState::Part3 => FrostDkgState::Finalized,
            _ => return Err(KrillError::DkgStateAlreadyFinalized),
        };

        self.storage().set_state(state).await?;

        Ok(state)
    }

    async fn part1(&self) -> KrillResult<FrostPart1Output<Self::DkgCipherSuite>> {
        let storage = self.storage();

        let current_state = storage.get_state().await?;

        if current_state != FrostDkgState::Initial {
            return Err(KrillError::InvalidDkgState(
                "Expected FROST Dkg to be `Initial` since no DKG has been performed at this point.",
            ));
        }

        let maximum_signers = storage.get_maximum_signers().await?;
        let minimum_signers = storage.get_minimum_signers().await?;
        let identifier = storage
            .get_identifier()
            .await?
            .decode::<Self::DkgCipherSuite>()?;

        let (secret, package) = frost_core::keys::dkg::part1(
            identifier,
            maximum_signers,
            minimum_signers,
            rand::thread_rng(),
        )
        .map_err(|error| KrillError::Part1KeyGenerationError(error.to_string()))?;

        storage.set_part1_package(secret, package.clone()).await?;
        self.frost_dkg_state_transition().await?;

        Ok(FrostPart1Output {
            identifier,
            package,
        })
    }

    async fn receive_part1(
        &self,
        identifier: frost_core::Identifier<Self::DkgCipherSuite>,
        package: frost_core::keys::dkg::round1::Package<Self::DkgCipherSuite>,
    ) -> KrillResult<()> {
        let state = self.storage().get_state().await?;
        let maximum_signers = self.storage().get_maximum_signers().await?;
        let party_count = self.storage().part1_received_packages_count().await?;

        if state != FrostDkgState::Part1 {
            return Err(KrillError::InvalidDkgState(
                "Expected FROST Dkg to be `Part1` since no DKG has been performed at this point.",
            ));
        }

        if party_count >= maximum_signers as usize {
            return Err(KrillError::Part1MaximumPartiesReached)?;
        }

        self.storage()
            .add_part1_received_package(identifier, package)
            .await?;

        // +2 where:
        // +1 here so that no new database query is made for a count of all parties
        // +1 since current party is also part of the DKG
        if party_count + 2 == maximum_signers as usize {
            self.frost_dkg_state_transition().await?;
        }

        Ok(())
    }

    async fn send_part1(&self) -> KrillResult<FrostPart1Output<Self::DkgCipherSuite>> {
        let storage = self.storage();
        let identifier = storage
            .get_identifier()
            .await?
            .decode::<Self::DkgCipherSuite>()?;
        let part_1_package = storage.get_part1_public_package().await?;

        Ok(FrostPart1Output {
            identifier,
            package: part_1_package,
        })
    }

    async fn part2(&self) -> KrillResult<crate::FrostPart2Output<Self::DkgCipherSuite>> {
        let state = self.state().await?;
        let identifier = self
            .storage()
            .get_identifier()
            .await?
            .decode::<Self::DkgCipherSuite>()?;

        if state != FrostDkgState::Part2 {
            return Err(KrillError::InvalidFrostDkgState(state.to_string()));
        }

        let part1_packages = self.storage().get_all_part1_received_packages().await?;
        let part1_secret = self.storage().get_part1_secret_package().await?;

        let (part2_secret, part2_packages) =
            frost_core::keys::dkg::part2(part1_secret, &part1_packages)
                .map_err(|error| KrillError::Part2KeyGenerationError(error.to_string()))?;

        self.storage()
            .set_part2_package(part2_secret, part2_packages.clone())
            .await?;

        Ok(FrostPart2Output {
            identifier,
            packages: part2_packages,
        })
    }

    async fn receive_part2(
        &self,
        identifier: frost_core::Identifier<Self::DkgCipherSuite>,
        package: frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>,
    ) -> KrillResult<()> {
        let state = self.storage().get_state().await?;
        let maximum_signers = self.storage().get_maximum_signers().await?;
        let party_count = self.storage().part2_received_packages_count().await?;

        if state != FrostDkgState::Part2 {
            return Err(KrillError::InvalidDkgState(
                "Expected FROST Dkg to be `Part2` since no DKG has been performed at this point.",
            ));
        }

        if party_count >= maximum_signers as usize {
            return Err(KrillError::Part2MaximumPartiesReached)?;
        }

        if state != FrostDkgState::Part2 {
            return Err(KrillError::InvalidDkgState(
                "Expected FROST Dkg to be `Part2` since no DKG has been performed at this point.",
            ));
        }
        self.storage()
            .add_part2_received_package(identifier, package)
            .await?;

        // +2 where:
        // +1 here so that no new database query is made for a count of all parties
        // +1 since current party is also part of the DKG
        if party_count + 2 == maximum_signers as usize {
            self.frost_dkg_state_transition().await?;
        }

        Ok(())
    }

    async fn send_part2(
        &self,
        identifier: &frost_core::Identifier<Self::DkgCipherSuite>,
    ) -> KrillResult<Option<frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>>> {
        self.storage().get_part2_package(identifier).await
    }

    async fn part3(&self) -> KrillResult<FrostKeypairData> {
        let state = self.storage().get_state().await?;

        if state != FrostDkgState::Part3 {
            return Err(KrillError::InvalidDkgState(
                "Expected FROST Dkg to be `Part3` since no DKG has been performed at this point.",
            ));
        }

        let mut part2_secret = self.storage().get_part2_secret().await?;
        let part1_packages = self.storage().get_all_part1_received_packages().await?;
        let part2_packages = self.storage().get_all_part2_received_packages().await?;

        let (secret, public_package) =
            frost_core::keys::dkg::part3(&part2_secret, &part1_packages, &part2_packages)
                .map_err(|error| KrillError::Part3Finalize(error.to_string()))?;

        let storage = self.storage();
        let identifier = storage.get_identifier().await?;
        let maximum_signers = storage.get_maximum_signers().await?;
        let minimum_signers = storage.get_minimum_signers().await?;
        let participants = part2_packages
            .keys()
            .map(|key| FrostIdentifier::encode(key))
            .collect::<Vec<FrostIdentifier>>();

        part2_secret.zeroize();
        self.storage().clear_dkg_data().await?;

        self.frost_dkg_state_transition().await?;

        let secret = FrostSigningKeyPackage::encode(&secret)?;
        let public_package = FrostSigningPublicKeyPackage::encode(&public_package)?;

        Ok(FrostKeypairData {
            identifier,
            maximum_signers,
            minimum_signers,
            secret,
            public_package,
            participants,
        })
    }
}
