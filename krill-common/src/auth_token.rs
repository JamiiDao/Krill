use std::time::Duration;

use bitcode::{Decode, Encode};
use tai64::Tai64N;

#[cfg(feature = "random")]
use {
    crate::{Holder, KrillError, KrillResult, RandomBytes},
    core::fmt,
};

#[cfg(feature = "random")]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct AuthTokenDetails {
    holder: Holder,
    timestamp: [u8; Tai64N::BYTE_SIZE],
    expiry: Duration,
    retry: Duration,
}

#[cfg(feature = "random")]
impl AuthTokenDetails {
    pub const COOKIE_AUTH_TOKEN_IDENTIFIER: &str = "auth-token";

    pub const BYTE_32_LEN: usize = 32;

    pub const AUTH_TOKEN_LEN: usize = Tai64N::BYTE_SIZE + Self::BYTE_32_LEN;
    pub const AUTH_TOKEN_BUFFER: [u8; Self::AUTH_TOKEN_LEN] = [0u8; Self::AUTH_TOKEN_LEN];

    pub fn new(holder: Holder) -> Self {
        let now = Tai64N::now();
        Self {
            holder,
            timestamp: now.to_bytes(),
            expiry: Duration::from_hours(24),
            retry: Duration::from_secs(30),
        }
    }

    pub fn store_key(&self, token: [u8; Self::BYTE_32_LEN]) -> [u8; Self::AUTH_TOKEN_LEN] {
        let mut buffer = Self::AUTH_TOKEN_BUFFER;

        buffer[0..Tai64N::BYTE_SIZE].copy_from_slice(&self.expiry_as_timestamp().to_bytes());
        buffer[Tai64N::BYTE_SIZE..].copy_from_slice(&token);

        buffer
    }

    pub fn store_key_hex(&self, token: [u8; Self::BYTE_32_LEN]) -> String {
        faster_hex::hex_string_upper(&self.store_key(token))
    }

    pub fn store_key_bytes_to_hex(token: [u8; Self::AUTH_TOKEN_LEN]) -> String {
        faster_hex::hex_string_upper(&token)
    }

    pub fn decode_token(token: &str) -> KrillResult<[u8; Self::AUTH_TOKEN_LEN]> {
        let mut buffer = Self::AUTH_TOKEN_BUFFER;

        faster_hex::hex_decode(token.as_bytes(), &mut buffer)
            .or(Err(KrillError::InvalidAuthToken))?;

        Ok(buffer)
    }

    pub fn holder(&self) -> &Holder {
        &self.holder
    }

    pub fn timestamp_bytes(&self) -> [u8; Tai64N::BYTE_SIZE] {
        self.timestamp
    }

    /// If result is Unix EPOCH it makes it an error unless that is what you were expecting
    pub fn timestamp_formatted(&self) -> String {
        use time::OffsetDateTime;

        let timestamp = self.timestamp().to_system_time();

        let utc_time: OffsetDateTime = timestamp.into();

        utc_time.to_string()
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

    pub fn generate_token() -> [u8; Self::BYTE_32_LEN] {
        *RandomBytes::<32>::generate().take()
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

    pub fn auth_token_as_cookie(&self, cookie_auth_token: [u8; Self::BYTE_32_LEN]) -> String {
        format!(
            "{}={}; Path=/; HttpOnly;{} SameSite=Strict; Max-Age={}",
            Self::COOKIE_AUTH_TOKEN_IDENTIFIER,
            self.store_key_hex(cookie_auth_token),
            if cfg!(not(debug_assertions)) {
                " Secure;"
            } else {
                ""
            },
            self.expiry.as_secs()
        )
    }

    pub fn auth_token_as_cookie_raw(
        &self,
        cookie_auth_token: [u8; Self::AUTH_TOKEN_LEN],
        same_site: &str,
    ) -> String {
        format!(
            "{}={}; Path=/; HttpOnly;{} SameSite={same_site}; Max-Age={}",
            Self::COOKIE_AUTH_TOKEN_IDENTIFIER,
            Self::store_key_bytes_to_hex(cookie_auth_token),
            if cfg!(not(debug_assertions)) {
                " Secure;"
            } else {
                ""
            },
            self.expiry.as_secs()
        )
    }

    pub fn const_cmp(
        current: [u8; Self::AUTH_TOKEN_LEN],
        other: &[u8; Self::AUTH_TOKEN_LEN],
    ) -> bool {
        use subtle::ConstantTimeEq;

        current.ct_eq(other).into()
    }

    pub fn extract_32_byte_token(
        token: [u8; Self::AUTH_TOKEN_LEN],
    ) -> KrillResult<[u8; Self::BYTE_32_LEN]> {
        token[0..Self::BYTE_32_LEN]
            .try_into()
            .or(Err(KrillError::InvalidAuthToken))
    }
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
