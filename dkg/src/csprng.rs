use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha12Rng,
};
use zeroize::Zeroizing;

pub struct RandomBytes<const N: usize>(Zeroizing<[u8; N]>);

impl<const N: usize> RandomBytes<N> {
    pub fn generate() -> Self {
        let mut rng = ChaCha12Rng::from_os_rng();

        let mut buffer = Zeroizing::new([0u8; N]);
        rng.fill_bytes(&mut buffer.as_mut());

        Self(buffer)
    }

    pub fn expose(&self) -> &[u8; N] {
        &self.0
    }

    pub fn take(self) -> Zeroizing<[u8; N]> {
        self.0
    }

    pub fn hash(&self) -> blake3::Hash {
        blake3::hash(self.expose())
    }
}
