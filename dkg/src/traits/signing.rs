use std::{collections::BTreeMap, future::Future};

use frost_core::{
    round1::{SigningCommitments, SigningNonces},
    Ciphersuite,
};

use crate::SecureHashing;

pub trait FrostDkgSigning {
    type DkgGenericError: core::error::Error;
    type DkgCipherSuite: Ciphersuite;

    fn storage(
        &self,
    ) -> impl Future<
        Output = Result<
            impl FrostDkgSigningStorage<Self::DkgCipherSuite, Self::DkgGenericError>,
            Self::DkgGenericError,
        >,
    >;

    fn signal_signing(
        &self,
        message: impl SecureHashing,
        participants: &[frost_core::Identifier<Self::DkgCipherSuite>],
    ) -> impl Future<Output = Result<(), Self::DkgCipherSuite>>;

    fn round1(
        &self,
    ) -> impl Future<Output = Result<SigningCommitments<Self::DkgCipherSuite>, Self::DkgCipherSuite>>;
}

pub trait FrostDkgSigningStorage<C: Ciphersuite, E: core::error::Error> {
    fn set_signing_signal(
        &self,
        message: impl SecureHashing,
        participants: &[frost_core::Identifier<C>],
    ) -> impl Future<Output = Result<(), E>>;

    fn get_message(
        &self,
        message_hash: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<impl SecureHashing, E>>;

    fn all_messages(
        &self,
        message_hash: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<Vec<impl SecureHashing>, E>>;

    fn all_messages_hashes(
        &self,
        message_hash: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<Vec<impl AsRef<[u8]>>, E>>;

    fn set_round1(
        &self,
        nonces: SigningNonces<C>,
        commitments: SigningCommitments<C>,
    ) -> impl Future<Output = Result<(), E>>;

    fn get_set_round1_nonces(&self) -> impl Future<Output = Result<SigningNonces<C>, E>>;

    fn get_set_round1_commitments(
        &self,
        nonces: SigningNonces<C>,
    ) -> impl Future<Output = Result<BTreeMap<frost_core::Identifier<C>, SigningCommitments<C>>, E>>;
}
