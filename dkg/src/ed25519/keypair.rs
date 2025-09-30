use std::collections::HashMap;

use frost_ed25519::{
    keys::{KeyPackage as Ed25519KeyPackage, PublicKeyPackage as Ed25519PublicKeyPackage},
    round1::commit as ed25519_round1_commit,
    Identifier as Ed25519Identifier,
};

pub type DkgMessageHash = blake3::Hash;

pub trait DkgMessageStore {
    fn set(&self, key: DkgMessageHash, value: impl AsRef<[u8]>);

    fn remove(&self, message_hash: DkgMessageHash);
}

#[derive(Debug)]
pub struct DkgMessageToSign {}

#[derive(Debug)]
pub struct FrostEd25519Keypair {
    party_id: &'static str,
    signing_key: Ed25519KeyPackage,
    verifying_key: Ed25519PublicKeyPackage,
}

impl FrostEd25519Keypair {
    pub(crate) fn new(
        party_id: &'static str,
        signing_key: Ed25519KeyPackage,
        verifying_key: Ed25519PublicKeyPackage,
    ) -> Self {
        Self {
            party_id,
            signing_key,
            verifying_key,
        }
    }

    pub fn insert_message(
        &self,
        store: impl DkgMessageStore,
        key: impl AsRef<[u8]>,
        message: impl AsRef<[u8]>,
    ) {
        store.set(blake3::hash(key.as_ref()), message);
    }

    pub fn round1_signing(&self) {
        let mut rng = rand::rngs::OsRng;

        let (nonces, commitments) =
            ed25519_round1_commit(self.signing_key.signing_share(), &mut rng);
    }

    pub fn verifying_key(&self) -> &Ed25519PublicKeyPackage {
        &self.verifying_key
    }

    pub fn party_id(&self) -> &'static str {
        self.party_id
    }

    pub fn identifier(&self) -> Ed25519Identifier {
        Ed25519Identifier::derive(self.party_id.as_bytes()).unwrap()
    }
}
