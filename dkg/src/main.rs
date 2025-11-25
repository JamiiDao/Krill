use std::sync::OnceLock;

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

static FROST_ED25519_MEM_STORAGE: OnceLock<Arc<RwLock<FrostDkgMemStorage>>> = OnceLock::new();

fn main() {
    smol::block_on(async {
        FROST_ED25519_MEM_STORAGE
            .set(Arc::new(RwLock::new(FrostDkgMemStorage::init())))
            .expect("Mem Storage should be initialized at this stage");

        let ed25519_dkg = FrostEd25519Dkg::new();

        let ed25519_identifier = ed25519_dkg.generate_identifier().unwrap();
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
            DkgState::Initial
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
    })
}
