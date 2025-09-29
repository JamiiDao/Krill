use frost_ed25519::{
    keys::{KeyPackage as Ed25519KeyPackage, PublicKeyPackage as Ed25519PublicKeyPackage},
    Identifier as Ed25519Identifier,
};

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
