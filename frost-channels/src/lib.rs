mod ops;
pub use ops::*;

mod state;
pub use state::*;

mod errors;
pub use errors::*;

mod traits;
pub use traits::*;

mod csprng;
pub use csprng::*;

#[cfg(test)]
mod test {
    use super::*;

    use async_dup::Arc;
    use async_lock::RwLock;

    #[test]
    fn test_dkg_and_signing() {
        smol::block_on(async {
            let party1 = "alice@example";

            let party2 = "bob@example";

            let ed25519_dkg_party1 =
                FrostEd25519Dkg::new(Arc::new(RwLock::new(FrostEd25519DkgMemStorage::init())));

            {
                //Init party 1
                let ed25519_identifier = ed25519_dkg_party1.generate_identifier(party1).unwrap();
                ed25519_dkg_party1
                    .storage()
                    .await
                    .unwrap()
                    .set_identifier(ed25519_identifier)
                    .await
                    .unwrap();

                ed25519_dkg_party1
                    .storage()
                    .await
                    .unwrap()
                    .set_maximum_signers(2)
                    .await
                    .unwrap();

                ed25519_dkg_party1
                    .storage()
                    .await
                    .unwrap()
                    .set_minimum_signers(2)
                    .await
                    .unwrap();
            }

            let ed25519_dkg_party2 =
                FrostEd25519Dkg::new(Arc::new(RwLock::new(FrostEd25519DkgMemStorage::init())));

            {
                //Init party 2
                let ed25519_identifier = ed25519_dkg_party1.generate_identifier(party2).unwrap();
                ed25519_dkg_party2
                    .storage()
                    .await
                    .unwrap()
                    .set_identifier(ed25519_identifier)
                    .await
                    .unwrap();

                ed25519_dkg_party2
                    .storage()
                    .await
                    .unwrap()
                    .set_maximum_signers(2)
                    .await
                    .unwrap();

                ed25519_dkg_party2
                    .storage()
                    .await
                    .unwrap()
                    .set_minimum_signers(2)
                    .await
                    .unwrap();
            }

            let party1_identifier = ed25519_dkg_party1
                .storage()
                .await
                .unwrap()
                .get_identifier()
                .await
                .unwrap();

            let party2_identifier = ed25519_dkg_party2
                .storage()
                .await
                .unwrap()
                .get_identifier()
                .await
                .unwrap();

            {
                // Part1

                ed25519_dkg_party1.part1().await.unwrap();
                ed25519_dkg_party2.part1().await.unwrap();

                let party1_part1_package = ed25519_dkg_party1
                    .storage()
                    .await
                    .unwrap()
                    .get_part1_public_package()
                    .await
                    .unwrap();
                let party2_part1_package = ed25519_dkg_party2
                    .storage()
                    .await
                    .unwrap()
                    .get_part1_public_package()
                    .await
                    .unwrap();

                ed25519_dkg_party1
                    .receive_part1(party2_identifier, party2_part1_package)
                    .await
                    .unwrap();
                ed25519_dkg_party2
                    .receive_part1(party1_identifier, party1_part1_package)
                    .await
                    .unwrap();
            }

            {
                // Part2
                let party1 = ed25519_dkg_party1.part2().await.unwrap();
                let party2 = ed25519_dkg_party2.part2().await.unwrap();

                assert_eq!(party1.identifier, party1_identifier);
                assert_eq!(party2.identifier, party2_identifier);

                let send_to_party2 = ed25519_dkg_party1
                    .send_part2(&party2_identifier)
                    .await
                    .unwrap()
                    .unwrap();
                let send_to_party1 = ed25519_dkg_party2
                    .send_part2(&party1_identifier)
                    .await
                    .unwrap()
                    .unwrap();

                ed25519_dkg_party1
                    .receive_part2(party2_identifier, send_to_party1)
                    .await
                    .unwrap();
                ed25519_dkg_party2
                    .receive_part2(party1_identifier, send_to_party2)
                    .await
                    .unwrap();
            }

            // Part3
            let party1_keys_data = ed25519_dkg_party1.part3().await.unwrap();
            let party2_keys_data = ed25519_dkg_party2.part3().await.unwrap();

            let party1_signing = FrostGenericSigning::new(Arc::new(RwLock::new(
                FrostEd25519SigningStorage::init(party1_keys_data),
            )));
            let party2_signing = FrostGenericSigning::new(Arc::new(RwLock::new(
                FrostEd25519SigningStorage::init(party2_keys_data),
            )));

            let message = "Hello FROST!";
            let message_hash = *blake3::hash(message.as_bytes()).as_bytes();
            let participants = &[
                // party1_signing.identifier().await.unwrap(),
                party2_signing.identifier().await.unwrap(),
            ];

            {
                // Coordinator is also signer
                let signal_round1 = party1_signing
                    .signal_round1(message_hash, participants, true)
                    .await
                    .unwrap();
                assert!(
                    party1_signing
                        .get_coordinator_message(&message_hash)
                        .await
                        .unwrap()
                        .unwrap()
                        .state
                        == SigningState::Round1
                );

                let round1_commit = party2_signing.round1_commit(signal_round1).await.unwrap();
                let receive_round1_commit = party1_signing
                    .receive_round1_commit(round1_commit)
                    .await
                    .unwrap();

                assert!(receive_round1_commit == SigningState::Round2);

                let signing_package = party1_signing
                    .signing_package(&message_hash, true)
                    .await
                    .unwrap();

                let round2 = party2_signing.round2_commit(signing_package).await.unwrap();
                let receive_round2_shares =
                    party1_signing.receive_round2_commit(round2).await.unwrap();

                assert!(receive_round2_shares == SigningState::Aggregate);

                assert!(
                    party1_signing
                        .all_coordinator_messages()
                        .await
                        .unwrap()
                        .len()
                        == 1usize
                );
                assert!(party1_signing
                    .all_participant_messages()
                    .await
                    .unwrap()
                    .is_empty());

                assert!(party2_signing
                    .all_coordinator_messages()
                    .await
                    .unwrap()
                    .is_empty());
                assert!(
                    party2_signing
                        .all_participant_messages()
                        .await
                        .unwrap()
                        .len()
                        == 1usize
                );

                let aggregate_signature_data =
                    party1_signing.aggregate(message_hash).await.unwrap();
                assert!(party2_signing
                    .verify_and_remove(&aggregate_signature_data)
                    .await
                    .is_ok())
            }
        })
    }
}
