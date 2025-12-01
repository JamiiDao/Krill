mod ed25519;
use async_dup::Arc;
use async_lock::RwLock;
pub use ed25519::*;

mod state;
pub use state::*;

mod errors;
pub use errors::*;

mod traits;
pub use traits::*;

mod csprng;
pub use csprng::*;

fn main() {
    smol::block_on(async {
        let party1 = "alice@example";

        let party2 = "bob@example";

        let ed25519_dkg_party1 =
            FrostEd25519Dkg::new(Arc::new(RwLock::new(FrostDkgMemStorage::init())));

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

            ed25519_dkg_party1.part1().await.unwrap();
        }

        let ed25519_dkg_party2 =
            FrostEd25519Dkg::new(Arc::new(RwLock::new(FrostDkgMemStorage::init())));

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

            ed25519_dkg_party2.part1().await.unwrap();
        }

        let party1_identifier = ed25519_dkg_party1
            .storage()
            .await
            .unwrap()
            .get_identifier()
            .await
            .unwrap();
        let party1_part1_package = ed25519_dkg_party1
            .storage()
            .await
            .unwrap()
            .get_part1_public_package()
            .await
            .unwrap();

        let party2_identifier = ed25519_dkg_party2
            .storage()
            .await
            .unwrap()
            .get_identifier()
            .await
            .unwrap();
        let party2_part1_package = ed25519_dkg_party2
            .storage()
            .await
            .unwrap()
            .get_part1_public_package()
            .await
            .unwrap();

        {
            // Part1
            ed25519_dkg_party1
                .receive_part1(party2_identifier, party2_part1_package)
                .await
                .unwrap();
            ed25519_dkg_party2
                .receive_part1(party1_identifier, party1_part1_package)
                .await
                .unwrap();
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_initialization() {
        smol::block_on(async {
            let ed25519_dkg =
                FrostEd25519Dkg::new(Arc::new(RwLock::new(FrostDkgMemStorage::init())));

            let ed25519_identifier = ed25519_dkg.generate_identifier_random().unwrap();
            ed25519_dkg
                .storage()
                .await
                .unwrap()
                .set_identifier(ed25519_identifier)
                .await
                .unwrap();
            assert_eq!(
                ed25519_dkg
                    .storage()
                    .await
                    .unwrap()
                    .get_identifier()
                    .await
                    .unwrap(),
                ed25519_identifier
            );

            assert_eq!(
                ed25519_dkg
                    .storage()
                    .await
                    .unwrap()
                    .get_state()
                    .await
                    .unwrap(),
                FrostDkgState::Initial
            );

            ed25519_dkg
                .storage()
                .await
                .unwrap()
                .set_maximum_signers(2)
                .await
                .unwrap();
            assert_eq!(
                ed25519_dkg
                    .storage()
                    .await
                    .unwrap()
                    .get_maximum_signers()
                    .await
                    .unwrap(),
                2u16
            );

            ed25519_dkg
                .storage()
                .await
                .unwrap()
                .set_minimum_signers(2)
                .await
                .unwrap();
            assert_eq!(
                ed25519_dkg
                    .storage()
                    .await
                    .unwrap()
                    .get_minimum_signers()
                    .await
                    .unwrap(),
                2u16
            );

            ed25519_dkg.part1().await.unwrap();
            assert_eq!(
                ed25519_dkg
                    .storage()
                    .await
                    .unwrap()
                    .get_state()
                    .await
                    .unwrap(),
                FrostDkgState::Part1
            );

            assert!(ed25519_dkg
                .storage()
                .await
                .unwrap()
                .get_part1_secret_package()
                .await
                .is_ok());

            assert!(ed25519_dkg
                .storage()
                .await
                .unwrap()
                .get_part1_secret_package()
                .await
                .is_err());

            assert!(ed25519_dkg
                .storage()
                .await
                .unwrap()
                .get_part1_public_package()
                .await
                .is_ok());
        })
    }
}
