use core::fmt;

use crate::{KrillError, KrillResult};

use bitcode::{Decode, Encode};
use email_address::EmailAddress;

#[derive(Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Encode, Decode)]
pub enum UserRole {
    Superuser,
    Admin,
    #[default]
    Member,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Superuser => "superuser",
            Self::Admin => "administrator",
            Self::Member => "member",
        }
    }
}

#[derive(Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct Holder {
    user_display: String,
    email: String,
    role: UserRole,
    frost_id: Vec<u8>,
}

impl Holder {
    pub fn new_allow_local(email_address: &str) -> KrillResult<Self> {
        EmailAddress::parse_with_options(
            email_address,
            email_address::Options::default().with_no_minimum_sub_domains(),
        )
        .map_err(|error| KrillError::InvalidEmailAddress(error.to_string()))?;

        Ok(Self {
            email: email_address.to_string(),
            ..Default::default()
        })
    }

    pub fn new_with_tld(email_address: &str) -> KrillResult<Self> {
        EmailAddress::parse_with_options(
            email_address,
            email_address::Options::default().with_required_tld(),
        )
        .map_err(|error| KrillError::InvalidEmailAddress(error.to_string()))?;

        Ok(Self {
            email: email_address.to_string(),
            role: UserRole::default(),
            ..Default::default()
        })
    }

    pub fn set_superuser(mut self) -> Self {
        self.role = UserRole::Superuser;

        self
    }

    pub fn set_admin(mut self) -> Self {
        self.role = UserRole::Admin;

        self
    }

    pub fn set_member(mut self) -> Self {
        self.role = UserRole::Member;

        self
    }

    pub fn set_user_display(mut self, names: &str) -> Self {
        self.user_display = names.to_string();

        self
    }

    pub fn obfuscate_email(&self) -> String {
        let email_address = self.to_email_address_struct();

        let mut local = email_address
            .local_part()
            .chars()
            .take(2)
            .collect::<String>();
        (0..email_address.local_part().len()).for_each(|_| {
            local.push('*');
        });

        local + "*****" + email_address.domain()
    }

    pub fn to_email_address_struct(&self) -> EmailAddress {
        EmailAddress::new_unchecked(&self.email)
    }

    pub fn email_address(&self) -> &str {
        self.email.as_str()
    }

    pub fn user_display(&self) -> &str {
        self.user_display.as_str()
    }

    pub fn role(&self) -> UserRole {
        self.role
    }

    pub fn email_envelope_details(&self) -> String {
        format!("{} <{}>", self.user_display(), self.email_address())
    }
}

impl fmt::Display for Holder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.email_address(), self.role.as_str())
    }
}
