use std::{collections::BTreeMap, marker::PhantomData};

use frost_core::Ciphersuite;
use krill_common::{KrillError, KrillResult};
use zeroize::Zeroize;

use crate::{
    AggregateSignatureData, CoordinatorMessageData, FrostDistributedSigning, FrostIdentifier,
    FrostSignature, FrostSignatureShare, FrostSigningCommitments, FrostSigningNonces,
    FrostSigningPackage, FrostStorage, Message32ByteHash, ParticipantMessageData, Round1CommitData,
    Round2SigningData, SigningPackageData, SigningRound1RequestData, SigningState,
};

pub struct FrostGenericSigning<C: Ciphersuite + Send + Sync, S: FrostStorage<C>>(S, PhantomData<C>);

impl<C: Ciphersuite + Send + Sync, S: FrostStorage<C> + Clone> FrostGenericSigning<C, S> {
    pub fn new(storage: S) -> Self {
        Self(storage, PhantomData)
    }
}

impl<C, S> FrostDistributedSigning for FrostGenericSigning<C, S>
where
    C: Ciphersuite + Send + Sync,
    S: FrostStorage<C> + Clone,
{
    type DkgCipherSuite = C;

    fn storage(&self) -> impl FrostStorage<Self::DkgCipherSuite> {
        self.0.clone()
    }

    async fn signal_round1(
        &self,
        message_hash: Message32ByteHash,
        participants: &[frost_core::Identifier<Self::DkgCipherSuite>],
        is_signer: bool,
    ) -> KrillResult<SigningRound1RequestData> {
        let mut invalid_participants = Vec::<Vec<u8>>::default();
        let keypair_data = self.storage().get_keypair_data().await?;

        participants.iter().for_each(|participant| {
            if !self
                .storage()
                .is_valid_participant(participant, &keypair_data)
            {
                invalid_participants.push(participant.serialize());
            }
        });

        if !invalid_participants.is_empty() {
            return Err(KrillError::InvalidParticipants(invalid_participants));
        }

        let participants = participants
            .iter()
            .map(|participant| FrostIdentifier::encode(participant))
            .collect::<Vec<FrostIdentifier>>();

        let mut message_data = CoordinatorMessageData {
            is_signer,
            state: SigningState::default(),
            participants: participants.clone(),
            message_hash,
            nonces: Option::default(),
            signing_package: Option::default(),
            commitments: BTreeMap::default(),
            signature_shares: BTreeMap::default(),
        };

        if is_signer {
            let signing_share = keypair_data.secret.decode::<Self::DkgCipherSuite>()?;

            let (nonces, commitments) =
                frost_core::round1::commit(signing_share.signing_share(), &mut rand::rngs::OsRng);

            message_data
                .nonces
                .replace(FrostSigningNonces::encode::<Self::DkgCipherSuite>(&nonces)?);
            message_data.commitments.insert(
                keypair_data.identifier.clone(),
                FrostSigningCommitments::encode(&commitments)?,
            );
            message_data
                .participants
                .push(keypair_data.identifier.clone());
        }

        self.storage()
            .set_coordinator_message(&message_data)
            .await?;

        Ok(SigningRound1RequestData {
            message_hash,
            participants,
            coordinator: keypair_data.identifier,
        })
    }

    async fn round1_commit(
        &self,
        message: SigningRound1RequestData,
    ) -> KrillResult<crate::Round1CommitData> {
        let keypair_data = self.storage().get_keypair_data().await?;
        let message_hash = message.message_hash;

        let signing_share = keypair_data.secret.decode::<Self::DkgCipherSuite>()?;
        let (nonces, commitments) =
            frost_core::round1::commit(signing_share.signing_share(), &mut rand::rngs::OsRng);
        let nonces = FrostSigningNonces::encode(&nonces)?;
        let commitments = FrostSigningCommitments::encode(&commitments)?;

        let message_data = ParticipantMessageData {
            message_hash,
            participants: message.participants,
            coordinator: message.coordinator,
            round1: Some((nonces, commitments.clone())),
            signing_package: Option::None,
            round2: Option::None,
        };

        self.storage()
            .set_participant_message(&message_data)
            .await?;

        Ok(Round1CommitData {
            message_hash,
            identifier: keypair_data.identifier,
            commitments,
        })
    }

    async fn receive_round1_commit(
        &self,
        commit_data: Round1CommitData,
    ) -> KrillResult<SigningState> {
        let mut message_data_store = self.storage().get_coordinator_messages().await?;

        let message_data = message_data_store
            .get_mut(&commit_data.message_hash)
            .ok_or(KrillError::MessageToSignNotFound)?;

        if message_data.state != SigningState::Round1 {
            return Err(KrillError::ExpectedRound1SigningState {
                message_hash: commit_data.message_hash,
                state: message_data.state.as_str(),
            });
        }

        if !message_data
            .participants
            .iter()
            .any(|participant| participant == &commit_data.identifier)
        {
            return Err(KrillError::InvalidParticipant {
                message_hash: commit_data.message_hash,
                participant: commit_data.identifier.0.to_vec(),
            });
        }

        message_data
            .commitments
            .insert(commit_data.identifier, commit_data.commitments);

        if message_data.commitments.len() == message_data.participants.len() {
            message_data.state = SigningState::Round2
        }

        let state = message_data.state;

        self.storage().set_coordinator_message(message_data).await?;

        Ok(state)
    }

    async fn signing_package(
        &self,
        message_hash: &Message32ByteHash,
        is_signer: bool,
    ) -> KrillResult<SigningPackageData> {
        let mut keypair_data = self.storage().get_keypair_data().await?;

        let mut message_data_store = self.storage().get_coordinator_messages().await?;

        let message_data = message_data_store
            .get_mut(message_hash)
            .ok_or(KrillError::MessageToSignNotFound)?;

        let mut all_signing_commitments = BTreeMap::default();
        message_data
            .commitments
            .iter()
            .try_for_each(|(key, value)| {
                let identifier = key.decode::<Self::DkgCipherSuite>()?;
                let signing_commitments = value.decode::<Self::DkgCipherSuite>()?;
                all_signing_commitments.insert(identifier, signing_commitments);

                Ok::<_, KrillError>(())
            })?;
        let signing_package = frost_core::SigningPackage::<Self::DkgCipherSuite>::new(
            all_signing_commitments,
            message_hash,
        );

        message_data
            .signing_package
            .replace(FrostSigningPackage::encode(&signing_package)?);

        let nonces = message_data
            .nonces
            .as_ref()
            .map(|nonces| nonces.decode::<Self::DkgCipherSuite>())
            .transpose()?
            .ok_or(KrillError::Round1NoncesNotFound)?;

        if is_signer {
            let signature_share =
                frost_core::round2::sign(&signing_package, &nonces, &keypair_data.secret.decode()?)
                    .map_err(|error| {
                        KrillError::SigningRound2(
                            "Coordinator unable to sign Round2 yet it is a signer.".to_string()
                                + error.to_string().as_str(),
                        )
                    })?;

            message_data.signature_shares.insert(
                keypair_data.identifier,
                FrostSignatureShare::encode(&signature_share),
            );

            keypair_data.secret.zeroize();
        }

        self.storage().set_coordinator_message(message_data).await?;

        Ok(SigningPackageData {
            message_hash: *message_hash,
            signing_package: FrostSigningPackage::encode(&signing_package)?,
        })
    }

    async fn round2_commit(
        &self,
        signing_package_data: SigningPackageData,
    ) -> KrillResult<Round2SigningData> {
        let mut keypair_data = self.storage().get_keypair_data().await?;

        let mut message_data_store = self.storage().get_participant_messages().await?;

        let message_data = message_data_store
            .get_mut(&signing_package_data.message_hash)
            .ok_or(KrillError::MessageToSignNotFound)?;

        let signature_share = if let Some(signature_share) = message_data.round2.as_ref() {
            signature_share.clone()
        } else {
            let nonces = message_data
                .round1
                .as_ref()
                .map(|(nonces, _commitments)| nonces.decode::<Self::DkgCipherSuite>())
                .transpose()?
                .ok_or(KrillError::SigningRound1NoncesAndCommitmentsNotFound)?;
            message_data
                .signing_package
                .replace(signing_package_data.signing_package);
            let signing_package = message_data
                .signing_package
                .as_ref()
                .map(|value| value.decode::<Self::DkgCipherSuite>())
                .transpose()?
                .ok_or(KrillError::SigningPackageNotFound)?;

            let signature_share = frost_core::round2::sign(
                &signing_package,
                &nonces,
                &keypair_data.secret.decode::<Self::DkgCipherSuite>()?,
            )
            .map_err(|error| {
                KrillError::SigningRound2(
                    "Participant unable to sign Round2 yet it is a signer.".to_string()
                        + error.to_string().as_str(),
                )
            })?;

            let signature_share = FrostSignatureShare::encode(&signature_share);

            message_data.round2.replace(signature_share.clone());

            signature_share
        };

        let identifier = keypair_data.identifier;
        keypair_data.secret.zeroize();

        self.storage().set_participant_message(message_data).await?;

        Ok(Round2SigningData {
            message_hash: signing_package_data.message_hash,
            identifier,
            signature_share,
        })
    }

    async fn receive_round2_commit(
        &self,
        signing_share_data: Round2SigningData,
    ) -> KrillResult<SigningState> {
        let mut keypair_data = self.storage().get_keypair_data().await?;

        let mut message_data_store = self.storage().get_coordinator_messages().await?;

        let message_data = message_data_store
            .get_mut(&signing_share_data.message_hash)
            .ok_or(KrillError::MessageToSignNotFound)?;

        if message_data.state != SigningState::Round2 {
            return Err(KrillError::ExpectedRound2SigningState {
                message_hash: message_data.message_hash,
                state: message_data.state.as_str(),
            });
        }

        if !message_data
            .participants
            .iter()
            .any(|participant| participant == &signing_share_data.identifier)
        {
            return Err(KrillError::InvalidParticipant {
                message_hash: signing_share_data.message_hash,
                participant: signing_share_data.identifier.0.to_vec(),
            });
        }

        message_data.signature_shares.insert(
            signing_share_data.identifier,
            signing_share_data.signature_share,
        );

        if message_data.signature_shares.len() == message_data.participants.len() {
            message_data.state = SigningState::Aggregate;
        }

        keypair_data.secret.zeroize();

        self.storage().set_coordinator_message(message_data).await?;

        Ok(message_data.state)
    }

    async fn aggregate(
        &self,
        message_hash: Message32ByteHash,
    ) -> KrillResult<AggregateSignatureData> {
        let storage = self.storage();
        let mut keypair_data = storage.get_keypair_data().await?;

        let mut message_data_store = storage.get_coordinator_messages().await?;

        let message_data = message_data_store
            .get_mut(&message_hash)
            .ok_or(KrillError::MessageToSignNotFound)?;

        if message_data.state != SigningState::Aggregate {
            return Err(KrillError::ExpectedAggregateSigningState {
                message_hash: message_data.message_hash,
                state: message_data.state.as_str(),
            });
        }

        let signing_package = message_data
            .signing_package
            .take()
            .ok_or(KrillError::SigningPackageNotFound)?
            .decode::<Self::DkgCipherSuite>()?;

        let mut signature_shares = BTreeMap::default();
        message_data
            .signature_shares
            .iter()
            .try_for_each(|(key, value)| {
                let key = key.decode::<Self::DkgCipherSuite>()?;
                let value = value.decode::<Self::DkgCipherSuite>()?;

                signature_shares.insert(key, value);

                Ok::<_, KrillError>(())
            })?;
        let aggregate_signature = frost_core::aggregate(
            &signing_package,
            &signature_shares,
            &keypair_data
                .public_package
                .decode::<Self::DkgCipherSuite>()?,
        )
        .map_err(|error| KrillError::UnableToAggregateSignature(error.to_string()))?;

        let identifier = storage.get_identifier().await?;
        let participants = core::mem::take(&mut message_data.participants);
        let outcome = AggregateSignatureData {
            message_hash,
            aggregate_signature: FrostSignature::encode(&aggregate_signature)?,
            coordinator: identifier,
            participants,
        };

        keypair_data.secret.zeroize();

        Ok(outcome)
    }

    async fn verify(&self, aggregate_signature_data: &AggregateSignatureData) -> KrillResult<()> {
        let mut keypair_data = self.storage().get_keypair_data().await?;

        keypair_data
            .public_package
            .decode::<Self::DkgCipherSuite>()?
            .verifying_key()
            .verify(
                &aggregate_signature_data.message_hash,
                &aggregate_signature_data
                    .aggregate_signature
                    .decode::<Self::DkgCipherSuite>()?,
            )
            .map_err(|error| KrillError::InvalidAggregateSignature(error.to_string()))?;

        keypair_data.secret.zeroize();

        Ok(())
    }

    async fn verify_and_remove(
        &self,
        aggregate_signature_data: &AggregateSignatureData,
    ) -> KrillResult<()> {
        self.verify(aggregate_signature_data).await?;

        self.storage()
            .clear_participant_messages(&aggregate_signature_data.message_hash)
            .await
    }
}
