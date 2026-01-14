use core::fmt;
use std::collections::BTreeMap;

use bitcode::{Decode, Encode};

use crate::{
    FrostDkgState, FrostIdentifier, FrostRound1PublicPackage, FrostRound1SecretPackage,
    FrostRound2PublicPackage, FrostRound2SecretPackage,
};

#[derive(Debug, Encode, Decode, Default)]
pub struct FrostDkgData {
    pub identifier: Option<FrostIdentifier>,
    pub maximum_signers: u16,
    pub minimum_signers: u16,
    pub dkg_state: FrostDkgState,
    pub part1_secret: Option<FrostRound1SecretPackage>,
    pub part1_package: Option<FrostRound1PublicPackage>,
    pub received_part1_packages: BTreeMap<FrostIdentifier, FrostRound1PublicPackage>,
    pub part2_secret: Option<FrostRound2SecretPackage>,
    pub part2_package: BTreeMap<FrostIdentifier, FrostRound2PublicPackage>,
    pub received_part2_packages: BTreeMap<FrostIdentifier, FrostRound2PublicPackage>,
}

impl FrostDkgData {
    pub fn init() -> Self {
        Self {
            identifier: Option::default(),
            part1_secret: Option::default(),
            part1_package: Option::default(),
            received_part1_packages: BTreeMap::default(),
            part2_secret: Option::default(),
            part2_package: BTreeMap::default(),
            received_part2_packages: BTreeMap::default(),
            maximum_signers: 2u16,
            minimum_signers: 2u16,
            dkg_state: FrostDkgState::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum StoreKeyspace {
    FrostKeypair,
    CoordinatorMessages,
    ParticipantMessages,
    SignedMessages,
}

impl StoreKeyspace {
    pub fn to_str(&self) -> &str {
        match self {
            Self::FrostKeypair => "frost-signing-keypair",
            Self::CoordinatorMessages => "frost-signing-coordinator-messages",
            Self::ParticipantMessages => "frost-signing-participant-messages",
            Self::SignedMessages => "frost-signing-signed-messages",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum StoreKeys {
    Dkg,
    KeypairData,
}

impl StoreKeys {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Dkg => "frost-dkg-key",
            Self::KeypairData => "frost-signing-keypair-data-key",
        }
    }
}

impl fmt::Display for StoreKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_str())
    }
}
