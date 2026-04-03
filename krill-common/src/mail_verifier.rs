use std::time::Duration;

use bitcode::{Decode, Encode};

#[cfg(feature = "random")]
use crate::AuthTokenDetails;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct VerifyMailDetailsToUi {
    pub obsf_mail: String,
    pub timestamp: [u8; 12],
    pub expiry: String,
    pub retry: Duration,
    pub cookie: String,
}

#[cfg(feature = "random")]
impl From<(blake3::Hash, AuthTokenDetails)> for VerifyMailDetailsToUi {
    fn from(value: (blake3::Hash, AuthTokenDetails)) -> Self {
        let auth_token = value.0;
        let auth_token_details = value.1;

        let expiry = auth_token_details.expiry_formatted();

        Self {
            obsf_mail: auth_token_details.holder().obfuscate_email(),
            timestamp: auth_token_details.timestamp_bytes(),
            expiry,
            retry: auth_token_details.retry(),
            cookie: auth_token_details.auth_token_as_cookie(&auth_token),
        }
    }
}
