use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha12Rng,
};
use zeroize::Zeroizing;

pub const ALPHABET: [char; 36] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

pub struct RandomChars<const N: usize>(Zeroizing<[char; N]>);

impl<const N: usize> RandomChars<N> {
    pub fn generate() -> Self {
        let mut rng = ChaCha12Rng::from_os_rng();

        let mut buffer = Zeroizing::new(['\0'; N]);

        for c in buffer.iter_mut() {
            // Generate a random index between 0 and 35
            let idx = (rng.next_u32() % 36) as usize;
            *c = ALPHABET[idx];
        }

        Self(buffer)
    }

    pub fn expose(&self) -> &[char; N] {
        &self.0
    }

    pub fn take(self) -> Zeroizing<[char; N]> {
        self.0
    }

    pub fn as_string(&self) -> Zeroizing<String> {
        let mut outcome = Zeroizing::new(String::with_capacity(N));

        self.0.into_iter().for_each(|char| {
            outcome.push(char);
        });

        outcome
    }

    pub fn as_string_passcode(&self) -> Zeroizing<String> {
        let mut outcome = Zeroizing::new(String::with_capacity(N));

        self.0.into_iter().enumerate().for_each(|(index, char)| {
            outcome.push(char);
            if index != 7 {
                outcome.push('-');
            }
        });

        outcome
    }

    pub fn hash(&self) -> blake3::Hash {
        blake3::hash(self.as_string().as_bytes())
    }

    pub fn const_cmp(&self, other: &str) -> bool {
        self.hash() == blake3::hash(other.as_bytes())
    }
}

pub struct RandomBytes<const N: usize>(Zeroizing<[u8; N]>);

impl<const N: usize> RandomBytes<N> {
    pub fn generate() -> Self {
        let mut rng = ChaCha12Rng::from_os_rng();

        let mut buffer = Zeroizing::new([0u8; N]);

        rng.fill_bytes(buffer.as_mut());

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

    pub fn const_cmp(&self, other: &[u8; N]) -> bool {
        self.hash() == blake3::hash(other)
    }
}
