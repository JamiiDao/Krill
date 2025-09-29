use std::collections::BTreeMap;

use frost_ed25519::{
    self as frost,
    keys::{
        dkg::round1::Package as Ed25519Round1Package,
        dkg::round1::SecretPackage as Ed25519Round1SecretPackage,
        dkg::round2::Package as Ed25519Round2Package,
        dkg::round2::SecretPackage as Ed25519Round2SecretPackage,
    },
    Identifier as Ed25519Identifier,
};

use crate::{DkgState, FrostEd25519Keypair};

type Round1PackagesStore = BTreeMap<Ed25519Identifier, Ed25519Round1Package>;
type Round2PackagesStore = BTreeMap<Ed25519Identifier, Ed25519Round2Package>;

#[derive(Debug)]
pub struct FrostEd25519Dkg {
    state: DkgState,
    // Identifier of current party
    party_id: &'static str,
    other_parties: Vec<&'static str>,
    round1_secret: Option<Ed25519Round1SecretPackage>,
    round2_secret: Option<Ed25519Round2SecretPackage>,
    round1_package: Option<Ed25519Round1Package>,
    received_round1_packages: Round1PackagesStore,
    computed_round2_packages: Round2PackagesStore,
    received_round2_packages: Round2PackagesStore,
    max_signers: u16,
    min_signers: u16,
}

impl FrostEd25519Dkg {
    /// Default to `5` maximum signers and `3` minimum signers
    pub fn new(party_id: &'static str) -> Self {
        let max_signers = 2u16;
        let min_signers = 1u16;

        Self {
            state: DkgState::Initial,
            party_id,
            other_parties: Vec::with_capacity(max_signers as usize),
            round1_secret: Option::default(),
            round2_secret: Option::default(),
            round1_package: Option::default(),
            received_round1_packages: BTreeMap::default(),
            computed_round2_packages: BTreeMap::default(),
            received_round2_packages: BTreeMap::default(),
            min_signers,
            max_signers,
        }
    }

    pub fn set_minimum_signers(mut self, minimum_signers: u16) -> Self {
        self.min_signers = minimum_signers;

        self
    }

    pub fn set_maximum_signers(mut self, maximum_signers: u16) -> Self {
        self.max_signers = maximum_signers;

        self
    }

    pub fn add_party(&mut self, party_identifier: &'static str) -> &mut Self {
        if self.state != DkgState::Initial {
            panic!("Max signers already reached. ")
        }

        self.other_parties.push(party_identifier);
        self.other_parties.dedup();

        if self.other_parties.len() == self.max_signers as usize {
            self.state = DkgState::Round1
        }

        self
    }

    pub fn add_parties(&mut self, party_identifier: &[&'static str]) -> &mut Self {
        party_identifier.iter().for_each(|party| {
            self.add_party(party);
        });

        self
    }

    /// Each participant generates their own identifier
    pub fn part1(&mut self) -> &mut Self {
        let rng = rand::rngs::OsRng;

        let identifier = Self::get_identifier(self.party_id);

        let (round1_secret, round1_package) =
            frost::keys::dkg::part1(identifier, self.max_signers, self.min_signers, rng).unwrap();

        self.round1_secret.replace(round1_secret);
        self.round1_package.replace(round1_package);

        self
    }

    pub fn part2(&mut self) -> &mut Self {
        let round1_secret = self.round1_secret.take().unwrap();

        let (round2_secret_package, mut round2_packages) =
            frost::keys::dkg::part2(round1_secret, &self.received_round1_packages).unwrap();

        self.round2_secret.replace(round2_secret_package);
        self.computed_round2_packages.append(&mut round2_packages);

        self
    }

    pub fn part3(&mut self) -> FrostEd25519Keypair {
        if self.state != DkgState::Round3 {
            panic!("Expected state to be RoundThree at this stage")
        }
        let round2_secret = self.round2_secret.take().unwrap();

        let (signing_key, verifying_key) = frost::keys::dkg::part3(
            &round2_secret,
            &self.received_round1_packages,
            &self.received_round2_packages,
        )
        .unwrap();

        FrostEd25519Keypair::new(self.party_id(), signing_key, verifying_key)
    }

    pub fn get_identifier(party_identifier: &'static str) -> Ed25519Identifier {
        Ed25519Identifier::derive(party_identifier.as_bytes()).unwrap()
    }

    pub fn add_received_round1_package(
        &mut self,
        party_identifier: &'static str,
        package: Ed25519Round1Package,
    ) -> &mut Self {
        if !self.party_is_authorized(party_identifier) {
            panic!("party is not authorized for participation")
        }

        if self.state != DkgState::Round1 {
            panic!("Expected state to be RoundOne at this stage")
        }

        self.received_round1_packages
            .insert(Self::get_identifier(party_identifier), package);

        if self.received_round1_packages.len() == (self.max_signers - 1) as usize {
            self.state = DkgState::Round2
        }

        self
    }

    pub fn party_is_authorized(&self, party_id: &'static str) -> bool {
        self.other_parties.contains(&party_id)
    }

    pub fn add_received_round2_package(
        &mut self,
        party_that_transmitted: &'static str,
        package: Ed25519Round2Package,
    ) -> &mut Self {
        if !self.party_is_authorized(party_that_transmitted) {
            panic!("party is not authorized for participation")
        }

        if self.state != DkgState::Round2 {
            panic!("Expected state to be RoundTwo at this stage")
        }
        self.received_round2_packages
            .insert(Self::get_identifier(party_that_transmitted), package);

        if self.received_round2_packages.len() == (self.max_signers - 1) as usize {
            self.state = DkgState::Round3
        }

        self
    }

    pub fn transmit_round2_package(
        &self,
        party_identifier: &'static str,
    ) -> Option<&Ed25519Round2Package> {
        self.computed_round2_packages
            .get(&Self::get_identifier(party_identifier))
    }

    pub fn party_id(&self) -> &'static str {
        self.party_id
    }

    pub fn identifier(&self) -> Ed25519Identifier {
        Self::get_identifier(self.party_id)
    }

    pub fn round1_package(&self) -> Option<&Ed25519Round1Package> {
        self.round1_package.as_ref()
    }

    pub fn round2_package(&self) -> &Round2PackagesStore {
        &self.computed_round2_packages
    }
}
