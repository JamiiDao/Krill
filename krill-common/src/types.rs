use core::fmt;

use bitcode::{Decode, Encode};

use crate::{KrillError, KrillResult};

pub type Message32ByteHash = [u8; 32];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub enum ServerOutcome<T> {
    Success(T),
    Failure(String),
}

impl<'a, T: Encode + Decode<'a>> ServerOutcome<T> {
    pub fn encode(outcome: KrillResult<T>) -> Vec<u8> {
        let outcome = match outcome {
            Ok(value) => Self::Success(value),
            Err(error) => Self::Failure(error.to_string()),
        };

        bitcode::encode(&outcome)
    }

    pub fn decode(bytes: &'a [u8]) -> KrillResult<ServerOutcome<T>> {
        bitcode::decode(bytes).or(Err(KrillError::UnableToDeserializeServerOutcome))
    }
}

#[derive(Encode, Decode, Clone)]
pub struct Blake3BytesRedacted([u8; 32]);

impl Blake3BytesRedacted {
    #[cfg(feature = "random")]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "random")]
impl Default for Blake3BytesRedacted {
    fn default() -> Self {
        Self(*crate::RandomBytes::<32>::generate().hash().as_bytes())
    }
}

impl fmt::Debug for Blake3BytesRedacted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blake3BytesRedacted[Redacted]")
    }
}

impl PartialEq for Blake3BytesRedacted {
    fn eq(&self, other: &Self) -> bool {
        let self_ = blake3::Hash::from_bytes(self.0);
        let other_ = blake3::Hash::from_bytes(other.0);

        self_ == other_
    }
}

impl Eq for Blake3BytesRedacted {}

// PartialEq, Eq, PartialOrd, Ord, Clone,

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Encode, Decode)]
pub enum UserRole {
    Administrator,
    Member,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Administrator => "administrator",
            Self::Member => "member",
        }
    }
}
