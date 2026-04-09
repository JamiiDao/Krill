use {
    crate::backend::SERVER_ORG_INFO,
    dioxus::server::{ServerFnError, ServerFnResult},
    krill_common::{AuthTokenDetails, KrillError, OrganizationInfo},
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

    pub fn parse_token(token: &str) -> ServerFnResult<[u8; AuthTokenDetails::AUTH_TOKEN_LEN]> {
        AuthTokenDetails::decode_token(token).map_err(|error| ServerFnError::ServerError {
            message: error.to_string(),
            code: 400,
            details: None,
        })
    }
}
