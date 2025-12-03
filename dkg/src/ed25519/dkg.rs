use frost_ed25519::{self as frost, Ed25519Sha512, Identifier as Ed25519Identifier};
use zeroize::Zeroize;

use crate::{
    FrostDkg, FrostDkgEd25519Storage, FrostDkgError, FrostDkgState, FrostDkgStorage,
    FrostPart1Output, FrostPart2Output, FrostPart3Output, RandomBytes,
};

pub struct Ed25519Sha512IdentifierGenerator;

impl Ed25519Sha512IdentifierGenerator {
    pub fn hashed_identifier(
        identifier: impl AsRef<[u8]>,
    ) -> Result<frost_core::Identifier<Ed25519Sha512>, FrostDkgError> {
        let identifier_bytes = *blake3::hash(identifier.as_ref()).as_bytes();

        let scalar_data = u128::from_le_bytes(identifier_bytes[0..16].try_into().or(Err(
            FrostDkgError::ToByteArray("Unable to cast the slice tto a [0u8;16] byte array"),
        ))?);

        Ed25519Identifier::new(scalar_data.into())
            .or(Err(FrostDkgError::IdentifierDerivationNotSupported))
    }

    pub fn random_identifier() -> Result<Ed25519Identifier, FrostDkgError> {
        let identifier = RandomBytes::<32>::generate();
        Ed25519Identifier::derive(&*identifier.take())
            .or(Err(FrostDkgError::IdentifierDerivationNotSupported))
    }
}

#[derive(Default)]
pub struct FrostEd25519Dkg<S: FrostDkgEd25519Storage>(S);

impl<S: FrostDkgEd25519Storage> FrostEd25519Dkg<S> {
    pub fn new(storage: S) -> Self {
        Self(storage)
    }
}

impl<S: FrostDkgEd25519Storage + Clone> FrostDkg for FrostEd25519Dkg<S> {
    type DkgCipherSuite = Ed25519Sha512;
    type DkgGenericError = FrostDkgError;

    async fn storage(
        &self,
    ) -> Result<
        impl FrostDkgStorage<Self::DkgCipherSuite, Self::DkgGenericError>,
        Self::DkgGenericError,
    > {
        Ok(self.0.clone())
    }

    async fn state(&self) -> Result<FrostDkgState, Self::DkgGenericError> {
        self.storage().await?.get_state().await
    }

    fn generate_identifier(
        &self,
        identifier: impl AsRef<[u8]>,
    ) -> Result<frost_core::Identifier<Self::DkgCipherSuite>, Self::DkgGenericError> {
        Ed25519Sha512IdentifierGenerator::hashed_identifier(identifier.as_ref())
    }

    fn generate_identifier_random(&self) -> Result<Ed25519Identifier, FrostDkgError> {
        Ed25519Sha512IdentifierGenerator::random_identifier()
    }

    async fn frost_dkg_state_transition(&self) -> Result<FrostDkgState, Self::DkgGenericError> {
        let current_state = self.storage().await?.get_state().await?;

        let state = match current_state {
            FrostDkgState::Initial => FrostDkgState::Part1,
            FrostDkgState::Part1 => FrostDkgState::Part2,
            FrostDkgState::Part2 => FrostDkgState::Part3,
            FrostDkgState::Part3 => FrostDkgState::Finalized,
            _ => return Err(FrostDkgError::DkgStateAlreadyFinalized),
        };

        self.storage().await?.set_state(state).await?;

        Ok(state)
    }

    async fn part1(&self) -> Result<FrostPart1Output<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let storage = self.storage().await?;

        let current_state = storage.get_state().await?;

        if current_state != FrostDkgState::Initial {
            return Err(FrostDkgError::InvalidDkgState(
                "Expected FROST Dkg to be `Initial` since no DKG has been performed at this point.",
            ));
        }

        let maximum_signers = storage.get_maximum_signers().await?;
        let minimum_signers = storage.get_minimum_signers().await?;
        let identifier = storage.get_identifier().await?;

        let (secret, package) = frost::keys::dkg::part1(
            identifier,
            maximum_signers,
            minimum_signers,
            rand::thread_rng(),
        )
        .map_err(|error| FrostDkgError::Part1KeyGenerationError(error.to_string()))?;

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
    ) -> Result<(), Self::DkgGenericError> {
        let state = self.storage().await?.get_state().await?;
        let maximum_signers = self.storage().await?.get_maximum_signers().await?;
        let party_count = self
            .storage()
            .await?
            .part1_received_packages_count()
            .await?;

        if state != FrostDkgState::Part1 {
            return Err(FrostDkgError::InvalidDkgState(
                "Expected FROST Dkg to be `Part1` since no DKG has been performed at this point.",
            ));
        }

        if party_count >= maximum_signers as usize {
            return Err(FrostDkgError::Part1MaximumPartiesReached)?;
        }

        self.storage()
            .await?
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

    async fn send_part1(
        &self,
    ) -> Result<FrostPart1Output<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let identifier = self.storage().await?.get_identifier().await?;
        let part_1_package = self.storage().await?.get_part1_public_package().await?;

        Ok(FrostPart1Output {
            identifier,
            package: part_1_package,
        })
    }

    async fn part2(
        &self,
    ) -> Result<crate::FrostPart2Output<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let state = self.state().await?;

        if state != FrostDkgState::Part2 {
            return Err(FrostDkgError::InvalidFrostDkgState(state.to_string()));
        }

        let part1_packages = self
            .storage()
            .await?
            .get_all_part1_received_packages()
            .await?;
        let part1_secret = self.storage().await?.get_part1_secret_package().await?;

        let (part2_secret, part2_packages) = frost::keys::dkg::part2(part1_secret, &part1_packages)
            .map_err(|error| FrostDkgError::Part2KeyGenerationError(error.to_string()))?;

        self.storage()
            .await?
            .set_part2_package(part2_secret, &part2_packages)
            .await?;

        let identifier = self.storage().await?.get_identifier().await?;

        Ok(FrostPart2Output {
            identifier,
            packages: part2_packages,
        })
    }

    async fn receive_part2(
        &self,
        identifier: frost_core::Identifier<Self::DkgCipherSuite>,
        package: frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>,
    ) -> Result<(), Self::DkgGenericError> {
        let state = self.storage().await?.get_state().await?;
        let maximum_signers = self.storage().await?.get_maximum_signers().await?;
        let party_count = self
            .storage()
            .await?
            .part2_received_packages_count()
            .await?;

        if state != FrostDkgState::Part2 {
            return Err(FrostDkgError::InvalidDkgState(
                "Expected FROST Dkg to be `Part2` since no DKG has been performed at this point.",
            ));
        }

        if party_count >= maximum_signers as usize {
            return Err(FrostDkgError::Part2MaximumPartiesReached)?;
        }

        if state != FrostDkgState::Part2 {
            return Err(FrostDkgError::InvalidDkgState(
                "Expected FROST Dkg to be `Part2` since no DKG has been performed at this point.",
            ));
        }
        self.storage()
            .await?
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
    ) -> Result<
        Option<frost_core::keys::dkg::round2::Package<Self::DkgCipherSuite>>,
        Self::DkgGenericError,
    > {
        self.storage().await?.get_part2_package(identifier).await
    }

    async fn part3(&self) -> Result<FrostPart3Output<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let state = self.storage().await?.get_state().await?;

        if state != FrostDkgState::Part3 {
            return Err(FrostDkgError::InvalidDkgState(
                "Expected FROST Dkg to be `Part3` since no DKG has been performed at this point.",
            ));
        }

        let mut part2_secret = self.storage().await?.get_part2_secret().await?;
        let part1_packages = self
            .storage()
            .await?
            .get_all_part1_received_packages()
            .await?;
        let part2_packages = self
            .storage()
            .await?
            .get_all_part2_received_packages()
            .await?;

        let (secret, public_package) =
            frost_core::keys::dkg::part3(&part2_secret, &part1_packages, &part2_packages)
                .map_err(|error| FrostDkgError::Part3Finalize(error.to_string()))?;

        let storage = self.storage().await?;
        let identifier = storage.get_identifier().await?;
        let maximum_signers = storage.get_maximum_signers().await?;
        let minimum_signers = storage.get_minimum_signers().await?;

        part2_secret.zeroize();
        self.storage().await?.clear().await?;

        self.frost_dkg_state_transition().await?;

        Ok(FrostPart3Output {
            identifier,
            maximum_signers,
            minimum_signers,
            secret,
            public_package,
        })
    }
}
