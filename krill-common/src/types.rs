use bitcode::{Decode, Encode};

#[cfg(feature = "random")]
use core::fmt;
#[cfg(feature = "random")]
use std::time::Duration;

#[cfg(feature = "random")]
use tai64::Tai64N;
#[cfg(feature = "random")]
use zeroize::Zeroizing;

#[cfg(feature = "random")]
use crate::RandomChars;

pub type Message32ByteHash = [u8; 32];

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, bitcode::Encode, bitcode::Decode)]
pub enum ServerConfigurationState {
    #[default]
    Uninitialized,
    LoginInitialization,
    Initialized,
}

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

#[cfg(feature = "random")]
pub struct AdminConfiguration {
    timestamp: Tai64N,
    secret: Option<RandomChars<8>>,
}

#[cfg(feature = "random")]
impl AdminConfiguration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timestamp(&self) -> &Tai64N {
        &self.timestamp
    }

    pub fn is_expired_after_60(&self) -> bool {
        self.is_expired(Duration::from_mins(60))
    }

    pub fn is_expired(&self, duration: Duration) -> bool {
        let now = Tai64N::now();

        now.duration_since(&self.timestamp)
            .is_ok_and(|value| value > duration)
    }

    pub fn secret(&self) -> Option<&RandomChars<8>> {
        self.secret.as_ref()
    }

    pub fn to_string(&self) -> Option<Zeroizing<String>> {
        self.secret.as_ref().map(|secret_chars| {
            let mut outcome = Zeroizing::new(String::with_capacity(8));
            secret_chars
                .expose()
                .iter()
                .for_each(|char| outcome.push(*char));

            outcome
        })
    }

    pub fn const_cmp(&self, other: &str) -> bool {
        if let Some(secret) = self.secret.as_ref() {
            secret.const_cmp(other)
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        drop(self.secret.take());
    }
}

#[cfg(feature = "random")]
impl Default for AdminConfiguration {
    fn default() -> Self {
        Self {
            timestamp: Tai64N::now(),
            secret: Some(RandomChars::generate()),
        }
    }
}

#[cfg(feature = "random")]
impl fmt::Debug for AdminConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdminConfiguration(Redacted 8 characters)",)
    }
}
