use {
    crate::backend::SERVER_ORG_INFO,
    dioxus::server::{ServerFnError, ServerFnResult},
    krill_common::{KrillError, OrganizationInfo},
};

pub struct ServerUtils;

impl ServerUtils {
    pub(crate) fn request_get_org() -> ServerFnResult<&'static OrganizationInfo> {
        SERVER_ORG_INFO
            .get()
            .ok_or(KrillError::ServerOrgInfoNotSet)
            .map_err(|error| {
                let error_message = "Error-OrgNotFound";

                tracing::error!("{error_message}. Error: `{error:?}`");

                ServerFnError::ServerError {
                    message: error_message.to_string() + ": Internal server error",
                    code: 500,
                    details: None,
                }
            })
    }

    pub fn to_hash(value: &str) -> ServerFnResult<blake3::Hash> {
        blake3::Hash::from_hex(&value).or(Err(ServerFnError::ServerError {
            message: "The auth token is invalid".to_string(),
            code: 400,
            details: None,
        }))
    }
}
