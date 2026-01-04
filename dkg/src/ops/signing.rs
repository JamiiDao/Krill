use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use frost_core::Ciphersuite;
use zeroize::Zeroize;

use crate::{
    AggregateSignatureData, FrostDistributedSigning, FrostDistributedSigningStorage, FrostDkgError,
    FrostKeyStore, Message32ByteHash, MessageData, ParticipantMessageData, Round1CommitData,
    Round2SigningData, SigningPackageData, SigningRound1RequestData, SigningState,
};

pub struct FrostGenericSigning<C: Ciphersuite, S: FrostDistributedSigningStorage<C, FrostDkgError>>(
    S,
    PhantomData<C>,
);

impl<C: Ciphersuite, S: FrostDistributedSigningStorage<C, FrostDkgError>>
    FrostGenericSigning<C, S>
{
    pub fn new(storage: S) -> Self {
        Self(storage, PhantomData)
    }
}

impl<C: Ciphersuite, S: FrostDistributedSigningStorage<C, FrostDkgError> + Clone>
    FrostDistributedSigning for FrostGenericSigning<C, S>
{
    type DkgCipherSuite = C;
    type DkgGenericError = FrostDkgError;

    async fn storage(
        &self,
    ) -> Result<
        impl FrostDistributedSigningStorage<Self::DkgCipherSuite, Self::DkgGenericError>,
        Self::DkgGenericError,
    > {
        Ok(self.0.clone())
    }

    async fn identifier(
        &self,
    ) -> Result<frost_core::Identifier<Self::DkgCipherSuite>, Self::DkgGenericError> {
        Ok(*self.storage().await?.get().await?.keys_data().identifier())
    }

    async fn signal_round1(
        &self,
        message_hash: Message32ByteHash,
        participants: &[frost_core::Identifier<Self::DkgCipherSuite>],
        is_signer: bool,
    ) -> Result<SigningRound1RequestData<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let storage = self.storage().await?;
        let read_store = storage.get().await?;
        let mut invalid_participants = Vec::<Vec<u8>>::default();

        participants.iter().for_each(|participant| {
            if !read_store.is_valid_participant(participant) {
                invalid_participants.push(participant.serialize());
            }
        });

        if !invalid_participants.is_empty() {
            return Err(FrostDkgError::InvalidParticipants(invalid_participants));
        }

        let mut message_data = MessageData::<Self::DkgCipherSuite> {
            is_signer,
            state: SigningState::default(),
            participants: participants.to_vec(),
            message_hash,
            nonces: Option::default(),
            signing_package: Option::default(),
            commitments: BTreeMap::default(),
            signature_shares: BTreeMap::default(),
        };

        let self_identifier = read_store.keys_data().identifier;

        if is_signer {
            let signing_share = read_store.keys_data().secret().signing_share();

            let (nonces, commitments) =
                frost_core::round1::commit(signing_share, &mut rand::rngs::OsRng);

            message_data.nonces.replace(nonces);
            message_data
                .commitments
                .insert(self_identifier, commitments);
            message_data.participants.push(self_identifier);
        }

        drop(read_store);

        let mut write_store = storage.set().await?;

        if !write_store.set_message(&message_hash, message_data) {
            Err(FrostDkgError::MessageToSignAlreadyExists(message_hash))
        } else {
            Ok(SigningRound1RequestData {
                message_hash,
                participants: participants.to_vec(),
                coordinator: self_identifier,
            })
        }
    }

    async fn round1_commit(
        &self,
        message: SigningRound1RequestData<C>,
    ) -> Result<crate::Round1CommitData<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let storage = self.storage().await?;
        let mut write_storage = storage.set().await?;

        let signing_share = write_storage.keys_data().secret().signing_share();
        let (nonces, commitments) =
            frost_core::round1::commit(signing_share, &mut rand::rngs::OsRng);

        let message_data = ParticipantMessageData::<C> {
            participants: message.participants,
            coordinator: message.coordinator,
            round1: Some((nonces, commitments)),
            signing_package: Option::None,
            round2: Option::None,
        };

        write_storage.set_participant_message(&message.message_hash, message_data);
        let self_identifier = write_storage.keys_data().identifier();

        Ok(Round1CommitData {
            message_hash: message.message_hash,
            identifier: *self_identifier,
            commitments,
        })
    }

    async fn receive_round1_commit(
        &self,
        commit_data: Round1CommitData<Self::DkgCipherSuite>,
    ) -> Result<SigningState, Self::DkgGenericError> {
        let storage = self.storage().await?;
        let mut write_storage = storage.set().await?;

        write_storage
            .get_mut_message(&commit_data.message_hash)
            .map(|message_data| {
                if message_data.state != SigningState::Round1 {
                    return Err(FrostDkgError::ExpectedRound1SigningState {
                        message_hash: commit_data.message_hash,
                        state: message_data.state,
                    });
                }

                if !message_data
                    .participants
                    .iter()
                    .any(|participant| participant == &commit_data.identifier)
                {
                    return Err(FrostDkgError::InvalidParticipant {
                        message_hash: commit_data.message_hash,
                        participant: commit_data.identifier.serialize(),
                    });
                }

                message_data
                    .commitments
                    .insert(commit_data.identifier, commit_data.commitments);

                if message_data.commitments.len() == message_data.participants.len() {
                    message_data.state = SigningState::Round2
                }

                Ok(message_data.state)
            })
            .transpose()?
            .ok_or(FrostDkgError::MessageToSignNotFound)
    }

    async fn signing_package(
        &self,
        message_hash: &Message32ByteHash,
        is_signer: bool,
    ) -> Result<SigningPackageData<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let storage = self.storage().await?;
        let read_storage = storage.get().await?;
        let mut key_package = read_storage.keys_data().secret().clone();
        let self_identifier = read_storage.keys_data().identifier;

        drop(read_storage);

        let mut write_storage = storage.set().await?;

        let signing_package = write_storage
            .get_mut_message(message_hash)
            .map(|message_data| {
                let commitments = core::mem::take(&mut message_data.commitments);

                let signing_package = frost_core::SigningPackage::<Self::DkgCipherSuite>::new(
                    commitments,
                    message_hash,
                );

                message_data
                    .signing_package
                    .replace(signing_package.clone());

                let nonces = message_data
                    .nonces
                    .as_ref()
                    .ok_or(FrostDkgError::Round1NoncesNotFound)?;

                if is_signer {
                    let signature_share =
                        frost_core::round2::sign(&signing_package, nonces, &key_package).map_err(
                            |error| {
                                FrostDkgError::SigningRound2(
                                    "Coordinator unable to sign Round2 yet it is a signer."
                                        .to_string()
                                        + error.to_string().as_str(),
                                )
                            },
                        )?;

                    key_package.zeroize();

                    message_data
                        .signature_shares
                        .insert(self_identifier, signature_share);
                }

                Ok(signing_package)
            })
            .transpose()?
            .ok_or(FrostDkgError::MessageToSignNotFound)?;

        Ok(SigningPackageData {
            message_hash: *message_hash,
            signing_package,
        })
    }

    async fn round2_commit(
        &self,
        signing_package_data: SigningPackageData<Self::DkgCipherSuite>,
    ) -> Result<Round2SigningData<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let storage = self.storage().await?;
        let read_storage = storage.get().await?;
        let self_identifier = read_storage.keys_data().identifier;
        let mut key_package = read_storage.keys_data().secret().clone();

        drop(read_storage);

        let mut write_storage = storage.set().await?;

        let signature_share = write_storage
            .get_participant_message_mut(&signing_package_data.message_hash)
            .map(|message_data| {
                if let Some(signature_share) = message_data.round2 {
                    Ok(signature_share)
                } else {
                    let (nonces, _commitments) = message_data
                        .round1
                        .as_ref()
                        .ok_or(FrostDkgError::SigningRound1NoncesAndCommitmentsNotFound)?;
                    message_data
                        .signing_package
                        .replace(signing_package_data.signing_package);
                    let signing_package = message_data
                        .signing_package
                        .as_ref()
                        .ok_or(FrostDkgError::SigningPackageNotFound)?;

                    let signature_share =
                        frost_core::round2::sign(signing_package, nonces, &key_package).map_err(
                            |error| {
                                FrostDkgError::SigningRound2(
                                    "Participant unable to sign Round2 yet it is a signer."
                                        .to_string()
                                        + error.to_string().as_str(),
                                )
                            },
                        )?;

                    message_data.round2.replace(signature_share);

                    Ok(signature_share)
                }
            })
            .ok_or(FrostDkgError::MessageToSignNotFound)??;

        key_package.zeroize();

        Ok(Round2SigningData {
            message_hash: signing_package_data.message_hash,
            identifier: self_identifier,
            signature_share,
        })
    }

    async fn receive_round2_commit(
        &self,
        signing_share_data: Round2SigningData<Self::DkgCipherSuite>,
    ) -> Result<SigningState, Self::DkgGenericError> {
        let storage = self.storage().await?;
        let mut write_storage = storage.set().await?;

        write_storage
            .get_mut_message(&signing_share_data.message_hash)
            .map(|message_data| {
                if message_data.state != SigningState::Round2 {
                    return Err(FrostDkgError::ExpectedRound2SigningState {
                        message_hash: message_data.message_hash,
                        state: message_data.state,
                    });
                }

                if !message_data
                    .participants
                    .iter()
                    .any(|participant| participant == &signing_share_data.identifier)
                {
                    return Err(FrostDkgError::InvalidParticipant {
                        message_hash: signing_share_data.message_hash,
                        participant: signing_share_data.identifier.serialize(),
                    });
                }

                message_data.signature_shares.insert(
                    signing_share_data.identifier,
                    signing_share_data.signature_share,
                );

                if message_data.signature_shares.len() == message_data.participants.len() {
                    message_data.state = SigningState::Aggregate;
                }

                Ok(message_data.state)
            })
            .ok_or(FrostDkgError::MessageToSignNotFound)?
    }

    async fn aggregate(
        &self,
        message_hash: Message32ByteHash,
    ) -> Result<AggregateSignatureData<Self::DkgCipherSuite>, Self::DkgGenericError> {
        let storage = self.storage().await?;

        let read_storage = storage.get().await?;

        let self_identifier = *read_storage.keys_data().identifier();
        let public_key_package = read_storage.keys_data().public_package().clone();

        drop(read_storage);

        let mut write_storage = storage.set().await?;

        write_storage
            .remove_message(&message_hash)
            .map(|mut message_data| {
                if message_data.state != SigningState::Aggregate {
                    return Err(FrostDkgError::ExpectedAggregateSigningState {
                        message_hash: message_data.message_hash,
                        state: message_data.state,
                    });
                }

                let signing_package = message_data
                    .signing_package
                    .take()
                    .ok_or(FrostDkgError::SigningPackageNotFound)?;
                let signature_shares = message_data.signature_shares;
                let aggregate_signature =
                    frost_core::aggregate(&signing_package, &signature_shares, &public_key_package)
                        .map_err(|error| {
                            FrostDkgError::UnableToAggregateSignature(error.to_string())
                        })?;

                let outcome = AggregateSignatureData {
                    message_hash,
                    aggregate_signature,
                    coordinator: self_identifier,
                    participants: message_data.participants,
                };

                Ok(outcome)
            })
            .ok_or(FrostDkgError::MessageToSignNotFound)?
    }

    async fn verify(
        &self,
        aggregate_signature_data: &AggregateSignatureData<Self::DkgCipherSuite>,
    ) -> Result<(), Self::DkgGenericError> {
        let storage = self.storage().await?;

        let read_storage = storage.get().await?;
        let public_key_package = read_storage.keys_data().public_package().clone();

        drop(read_storage);

        public_key_package
            .verifying_key()
            .verify(
                &aggregate_signature_data.message_hash,
                &aggregate_signature_data.aggregate_signature,
            )
            .map_err(|error| FrostDkgError::InvalidAggregateSignature(error.to_string()))
    }

    async fn verify_and_remove(
        &self,
        aggregate_signature_data: &AggregateSignatureData<Self::DkgCipherSuite>,
    ) -> Result<(), Self::DkgGenericError> {
        self.verify(aggregate_signature_data).await?;

        let storage = self.storage().await?;

        let mut write_storage = storage.set().await?;
        if !write_storage.remove_participant_message(&aggregate_signature_data.message_hash) {
            Err(FrostDkgError::UnableToRemoveValidSignedParticipantMessage)
        } else {
            Ok(())
        }
    }

    async fn all_coordinator_messages(
        &self,
    ) -> Result<BTreeMap<Message32ByteHash, MessageData<Self::DkgCipherSuite>>, Self::DkgGenericError>
    {
        Ok(self.storage().await?.get().await?.coordinator_messages())
    }

    async fn all_participant_messages(
        &self,
    ) -> Result<
        HashMap<Message32ByteHash, ParticipantMessageData<Self::DkgCipherSuite>>,
        Self::DkgGenericError,
    > {
        Ok(self.storage().await?.get().await?.participant_messages())
    }

    async fn get_coordinator_message(
        &self,
        message_hash: &Message32ByteHash,
    ) -> Result<Option<MessageData<Self::DkgCipherSuite>>, Self::DkgGenericError> {
        Ok(self
            .storage()
            .await?
            .get()
            .await?
            .get_message(message_hash)
            .cloned())
    }

    async fn get_participant_messages(
        &self,
        message_hash: &Message32ByteHash,
    ) -> Result<Option<ParticipantMessageData<Self::DkgCipherSuite>>, Self::DkgGenericError> {
        Ok(self
            .storage()
            .await?
            .get()
            .await?
            .get_participant_message(message_hash)
            .cloned())
    }
}
