use bitcode::{Decode, Encode};

pub type Message32ByteHash = [u8; 32];

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
