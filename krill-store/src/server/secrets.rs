use bitcode::{Decode, Encode};
use krill_common::{KrillError, KrillResult, RandomBytes, UserRole};

use crate::KrillStorage;

impl KrillStorage {
    pub const KEYSPACE_SERVER_SECRET: &str = "ServerSecret";
    pub const STORE_KEY_SERVER_SECRET: &str = "secret";

    pub async fn get_server_secret(&self) -> KrillResult<[u8; 32]> {
        let keyspace = self.secrets_keyspace();

        if let Some(secret) = self
            .get_op(keyspace.clone(), Self::STORE_KEY_SERVER_SECRET)
            .await?
            .map(|bytes| {
                bitcode::decode::<[u8; 32]>(&bytes).or(Err(KrillError::ServerSecretNotFound))
            })
            .transpose()?
        {
            Ok(secret)
        } else {
            let secret = *RandomBytes::<32>::generate().take();

            self.set_op(keyspace, Self::STORE_KEY_SERVER_SECRET, secret)
                .await?;

            Ok(secret)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, Copy)]
pub struct ServerCookie {
    pub data: ServerCookieData,
    pub hash: [u8; 32],
}

impl ServerCookie {
    pub const IDENTIFIER: &str = "sessionid";

    pub fn new_admininstrator() -> Self {
        Self::new(UserRole::Administrator)
    }

    pub fn new_member() -> Self {
        Self::new(UserRole::Member)
    }

    // Expiry is set to 30 days
    pub fn new(role: UserRole) -> Self {
        let data = ServerCookieData::new(role);

        let hash = *Self::hash(&data).as_bytes();

        Self { data, hash }
    }

    pub fn hash(data: &ServerCookieData) -> blake3::Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&[data.role as u8]);
        hasher.update(&data.issued);
        hasher.update(&data.random);

        hasher.finalize()
    }
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, Copy)]
pub struct ServerCookieData {
    pub role: UserRole,
    pub issued: [u8; 12],
    pub expiry: [u8; 12],
    pub random: [u8; 32],
}

impl ServerCookieData {
    // Expiry is set to 30 days
    pub fn new(role: UserRole) -> Self {
        let now = tai64::Tai64N::now();
        let issued = now.to_bytes();
        let expiry = (now + std::time::Duration::from_hours(24 * 30)).to_bytes();
        let random = *RandomBytes::<32>::generate().expose();

        Self {
            role,
            issued,
            expiry,
            random,
        }
    }
}
