use bitcode::{Decode, Encode};
use frost_core::Ciphersuite;
use zeroize::Zeroize;

use crate::{KrillError, KrillResult};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostIdentifier(pub Vec<u8>);

impl FrostIdentifier {
    pub fn encode<C: Ciphersuite>(identifier: &frost_core::Identifier<C>) -> Self {
        Self(identifier.serialize())
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::Identifier<C>> {
        frost_core::Identifier::<C>::deserialize(&self.0)
            .or(Err(KrillError::UnableToDeserializeFrostIdentifier))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Zeroize)]
pub struct FrostRound1SecretPackage(Vec<u8>);

impl FrostRound1SecretPackage {
    pub fn encode<C: Ciphersuite>(
        secret_package: &frost_core::keys::dkg::round1::SecretPackage<C>,
    ) -> KrillResult<Self> {
        secret_package.serialize().map(Self).or(Err(
            KrillError::UnableToSerializeFrostDkgRound1SecretPackage,
        ))
    }

    pub fn decode<C: Ciphersuite>(
        &self,
    ) -> KrillResult<frost_core::keys::dkg::round1::SecretPackage<C>> {
        frost_core::keys::dkg::round1::SecretPackage::<C>::deserialize(&self.0).or(Err(
            KrillError::UnableToDeserializeFrostDkgRound1SecretPackage,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostRound1PublicPackage(Vec<u8>);

impl FrostRound1PublicPackage {
    pub fn encode<C: Ciphersuite>(
        public_package: &frost_core::keys::dkg::round1::Package<C>,
    ) -> KrillResult<Self> {
        public_package.serialize().map(Self).or(Err(
            KrillError::UnableToSerializeFrostDkgRound1PublicPackage,
        ))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::keys::dkg::round1::Package<C>> {
        frost_core::keys::dkg::round1::Package::<C>::deserialize(&self.0).or(Err(
            KrillError::UnableToDeserializeFrostDkgRound1PublicPackage,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Zeroize)]
pub struct FrostRound2SecretPackage(Vec<u8>);

impl FrostRound2SecretPackage {
    pub fn encode<C: Ciphersuite>(
        secret_package: &frost_core::keys::dkg::round2::SecretPackage<C>,
    ) -> KrillResult<Self> {
        secret_package.serialize().map(Self).or(Err(
            KrillError::UnableToSerializeFrostDkgRound2SecretPackage,
        ))
    }

    pub fn decode<C: Ciphersuite>(
        &self,
    ) -> KrillResult<frost_core::keys::dkg::round2::SecretPackage<C>> {
        frost_core::keys::dkg::round2::SecretPackage::<C>::deserialize(&self.0).or(Err(
            KrillError::UnableToDeserializeFrostDkgRound2SecretPackage,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostRound2PublicPackage(Vec<u8>);

impl FrostRound2PublicPackage {
    pub fn encode<C: Ciphersuite>(
        public_package: &frost_core::keys::dkg::round2::Package<C>,
    ) -> KrillResult<Self> {
        public_package.serialize().map(Self).or(Err(
            KrillError::UnableToSerializeFrostDkgRound2PublicPackage,
        ))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::keys::dkg::round2::Package<C>> {
        frost_core::keys::dkg::round2::Package::<C>::deserialize(&self.0).or(Err(
            KrillError::UnableToDeserializeFrostDkgRound2PublicPackage,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Zeroize)]
pub struct FrostSigningKeyPackage(Vec<u8>);

impl FrostSigningKeyPackage {
    pub fn encode<C: Ciphersuite>(
        signing_key: &frost_core::keys::KeyPackage<C>,
    ) -> KrillResult<Self> {
        signing_key
            .serialize()
            .map(Self)
            .or(Err(KrillError::UnableToSerializeFrostSigningKeyPackage))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::keys::KeyPackage<C>> {
        frost_core::keys::KeyPackage::<C>::deserialize(&self.0)
            .or(Err(KrillError::UnableToDeserializeFrostSigningKeyPackage))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostSigningPublicKeyPackage(Vec<u8>);

impl FrostSigningPublicKeyPackage {
    pub fn encode<C: Ciphersuite>(
        public_key_package: &frost_core::keys::PublicKeyPackage<C>,
    ) -> KrillResult<Self> {
        public_key_package.serialize().map(Self).or(Err(
            KrillError::UnableToSerializeFrostSigningPublicKeyPackage,
        ))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::keys::PublicKeyPackage<C>> {
        frost_core::keys::PublicKeyPackage::<C>::deserialize(&self.0).or(Err(
            KrillError::UnableToDeserializeFrostSigningPublicKeyPackage,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Zeroize)]
pub struct FrostSigningNonces(Vec<u8>);

impl FrostSigningNonces {
    pub fn encode<C: Ciphersuite>(
        signing_nonces: &frost_core::round1::SigningNonces<C>,
    ) -> KrillResult<Self> {
        signing_nonces
            .serialize()
            .map(Self)
            .or(Err(KrillError::UnableToSerializeFrostSigningNonces))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::round1::SigningNonces<C>> {
        frost_core::round1::SigningNonces::<C>::deserialize(&self.0)
            .or(Err(KrillError::UnableToDeserializeFrostSigningNonces))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostSigningCommitments(Vec<u8>);

impl FrostSigningCommitments {
    pub fn encode<C: Ciphersuite>(
        signing_commitments: &frost_core::round1::SigningCommitments<C>,
    ) -> KrillResult<Self> {
        signing_commitments
            .serialize()
            .map(Self)
            .or(Err(KrillError::UnableToSerializeFrostSigningCommitments))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::round1::SigningCommitments<C>> {
        frost_core::round1::SigningCommitments::<C>::deserialize(&self.0)
            .or(Err(KrillError::UnableToDeserializeFrostSigningCommitments))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostSigningPackage(Vec<u8>);

impl FrostSigningPackage {
    pub fn encode<C: Ciphersuite>(
        signing_package: &frost_core::SigningPackage<C>,
    ) -> KrillResult<Self> {
        signing_package
            .serialize()
            .map(Self)
            .or(Err(KrillError::UnableToSerializeFrostSigningPackage))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::SigningPackage<C>> {
        frost_core::SigningPackage::<C>::deserialize(&self.0)
            .or(Err(KrillError::UnableToDeserializeFrostSigningPackage))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostSignatureShare(Vec<u8>);

impl FrostSignatureShare {
    pub fn encode<C: Ciphersuite>(signature_share: &frost_core::round2::SignatureShare<C>) -> Self {
        Self(signature_share.serialize())
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::round2::SignatureShare<C>> {
        frost_core::round2::SignatureShare::<C>::deserialize(&self.0)
            .or(Err(KrillError::UnableToDeserializeFrostSignatureShare))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct FrostSignature(Vec<u8>);

impl FrostSignature {
    pub fn encode<C: Ciphersuite>(signature: &frost_core::Signature<C>) -> KrillResult<Self> {
        signature
            .serialize()
            .map(Self)
            .or(Err(KrillError::UnableToSerializeFrostSignature))
    }

    pub fn decode<C: Ciphersuite>(&self) -> KrillResult<frost_core::Signature<C>> {
        frost_core::Signature::<C>::deserialize(&self.0)
            .or(Err(KrillError::UnableToDeserializeFrostSignature))
    }
}
