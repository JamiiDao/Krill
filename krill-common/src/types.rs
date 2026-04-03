#[cfg(feature = "random")]
use core::fmt;
#[cfg(feature = "random")]
use std::time::Duration;

#[cfg(feature = "random")]
use tai64::Tai64N;

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

#[cfg(feature = "random")]
pub struct AdminConfiguration {
    timestamp: Tai64N,
    secret: RandomChars<8>,
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

    pub fn secret(&self) -> &RandomChars<8> {
        &self.secret
    }

    pub fn secret_to_string(&self) -> String {
        self.secret.expose().iter().collect::<String>()
    }

    pub fn const_cmp(&self, other: &str) -> bool {
        self.secret.const_cmp(other)
    }

    pub fn clear(&mut self) {
        self.secret.zeroize_mem()
    }
}

#[cfg(feature = "random")]
impl Default for AdminConfiguration {
    fn default() -> Self {
        Self {
            timestamp: Tai64N::now(),
            secret: RandomChars::generate(),
        }
    }
}

#[cfg(feature = "random")]
impl fmt::Debug for AdminConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdminConfiguration(Redacted 8 characters)",)
    }
}
