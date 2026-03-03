use bitcode::{Decode, Encode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub struct OrganizationInfo {
    pub name: String,
    pub threshold: u16,
}
