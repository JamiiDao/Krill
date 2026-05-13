use bitcode::{Decode, Encode};
use dioxus::{prelude::*, Result as DioxusResult};

use krill_common::{KrillError, KrillResult, UserRole};

#[cfg(feature = "server")]
use {
    crate::backend::store,
    dioxus::fullstack::{Cookie, TypedHeader},
};

#[get("/dashboard-data",  header: TypedHeader<Cookie>)]
pub async fn dashboard_data() -> DioxusResult<Vec<u8>> {
    use krill_common::AuthTokenDetails;

    let cookie = header
        .get(AuthTokenDetails::COOKIE_AUTH_TOKEN_IDENTIFIER)
        .or_unauthorized("Unauthorized user")?;

    let token = AuthTokenDetails::decode_token(cookie)?;
    let get_auth_details = store()?
        .get_auth_token_checked(token)
        .await?
        .or_unauthorized("Unauthorized user")?;

    match get_auth_details.holder().role() {
        UserRole::Superuser => superuser_dashboard_details().await,
        UserRole::Admin => superuser_dashboard_details().await,
        UserRole::Member => superuser_dashboard_details().await,
    }
}

async fn superuser_dashboard_details() -> DioxusResult<Vec<u8>> {
    let outcome = DashboardData {
        user_role: UserRole::Superuser,
    };
    Ok(outcome.encode())
}

async fn admin_dashboard_details() -> DioxusResult<Vec<u8>> {
    let outcome = DashboardData {
        user_role: UserRole::Admin,
    };
    Ok(outcome.encode())
}
async fn member_dashboard_details() -> DioxusResult<Vec<u8>> {
    let outcome = DashboardData {
        user_role: UserRole::Member,
    };
    Ok(outcome.encode())
}

#[derive(Debug, Default, PartialEq, Eq, Encode, Decode)]
pub struct DashboardData {
    pub user_role: UserRole,
}

impl DashboardData {
    pub fn encode(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn decode(bytes: &[u8]) -> KrillResult<Self> {
        bitcode::decode::<Self>(bytes).or(Err(KrillError::Transmit(
            "Unable to decode DashboardData".to_string(),
        )))
    }
}
