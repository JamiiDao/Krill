use std::time::Duration;

use bitcode::{Decode, Encode};
use tai64::Tai64N;

#[cfg(feature = "random")]
use crate::{Holder, RandomBytes};

#[cfg(feature = "random")]
use core::fmt;

#[cfg(feature = "random")]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct AuthTokenDetails {
    holder: Holder,
    timestamp: [u8; Tai64N::BYTE_SIZE],
    expiry: Duration,
    retry: Duration,
}

#[cfg(feature = "random")]
impl fmt::Debug for AuthTokenDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthTokenDetails")
            .field("holder", &self.holder)
            .field("timestamp", &self.timestamp_formatted())
            .field("expiry", &humantime::format_duration(self.expiry()))
            .field("retry", &humantime::format_duration(self.retry()))
            .finish()
    }
}

#[cfg(feature = "random")]
impl fmt::Display for AuthTokenDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthTokenDetails")
            .field("holder", &self.holder.to_string())
            .field("timestamp", &self.timestamp_formatted())
            .field("expiry", &humantime::format_duration(self.expiry()))
            .field("retry", &humantime::format_duration(self.retry()))
            .finish()
    }
}

#[cfg(feature = "random")]
impl AuthTokenDetails {
    pub const COOKIE_AUTH_TOKEN_IDENTIFIER: &str = "auth-token";

    pub fn new(holder: Holder) -> Self {
        let now = Tai64N::now();
        Self {
            holder,
            timestamp: now.to_bytes(),
            expiry: Duration::from_hours(24),
            retry: Duration::from_secs(30),
        }
    }

    pub fn store_key(&self, token: &blake3::Hash) -> [u8; Tai64N::BYTE_SIZE + blake3::KEY_LEN] {
        let mut buffer = [0u8; Tai64N::BYTE_SIZE + blake3::KEY_LEN];

        buffer[0..Tai64N::BYTE_SIZE].copy_from_slice(&self.expiry_as_timestamp().to_bytes());
        buffer[Tai64N::BYTE_SIZE..].copy_from_slice(token.as_bytes());

        buffer
    }

    pub fn holder(&self) -> &Holder {
        &self.holder
    }

    pub fn timestamp_bytes(&self) -> [u8; Tai64N::BYTE_SIZE] {
        self.timestamp
    }

    /// If result is Unix EPOCH it makes it an error unless that is what you were expecting
    pub fn timestamp_formatted(&self) -> String {
        let timestamp = self
            .timestamp()
            .duration_since(&Tai64N::UNIX_EPOCH)
            .unwrap_or_default();

        humantime::format_duration(timestamp).to_string()
    }

    /// If result is Unix EPOCH it makes it an error unless that is what you were expecting
    pub fn timestamp(&self) -> Tai64N {
        Self::to_tai64_timestamp(self.timestamp)
    }

    /// If result is Unix EPOCH it makes it an error unless that is what you were expecting
    pub fn to_tai64_timestamp(bytes: [u8; Tai64N::BYTE_SIZE]) -> Tai64N {
        Tai64N::try_from(bytes).unwrap_or(Tai64N::UNIX_EPOCH) //Not expected to unwrap since this is not user input
    }

    pub fn expiry_as_timestamp(&self) -> Tai64N {
        Self::to_tai64_timestamp((self.timestamp() + self.expiry).to_bytes())
    }

    pub fn expiry(&self) -> Duration {
        self.expiry
    }

    pub fn expiry_formatted(&self) -> String {
        format!(
            "{:.2} hours",
            self.expiry().as_secs() as f64 / (60f64 * 60f64)
        )
    }

    pub fn retry(&self) -> Duration {
        self.retry
    }

    pub fn generate_token() -> blake3::Hash {
        RandomBytes::<32>::generate().hash()
    }

    pub fn can_resend(&self) -> bool {
        let now = Tai64N::now();

        now >= self.timestamp()
    }

    pub fn is_expired(&self) -> bool {
        let as_tai = self.timestamp();

        if as_tai == Tai64N::UNIX_EPOCH {
            return true;
        }

        let expiry = as_tai + self.expiry();

        Tai64N::now() > expiry
    }

    pub fn auth_token_as_cookie(&self, cookie_auth_token: &blake3::Hash) -> String {
        format!(
            "{}={}; Path=/; HttpOnly;{} SameSite=Strict; Max-Age={}",
            Self::COOKIE_AUTH_TOKEN_IDENTIFIER,
            cookie_auth_token,
            if cfg!(not(debug_assertions)) {
                " Secure;"
            } else {
                ""
            },
            self.expiry.as_secs()
        )
    }
}
