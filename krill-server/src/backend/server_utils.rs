use axum::body::Body as AxumBody;
use axum::http::HeaderValue;
use dioxus::{fullstack::response::Response, prelude::*};
use krill_common::{AuthTokenDetails, KrillError, OrganizationInfo};

use crate::backend::SERVER_ORG_INFO;

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

#[derive(Debug)]
pub struct MediaTypeHttp(Response<AxumBody>);

impl MediaTypeHttp {
    pub fn builder(bytes: Vec<u8>) -> ServerFnResult<Self> {
        let media_type = wasm_toolkit::WasmToolkitCommon::media_type(bytes.as_slice()).to_string();

        let mut res = Response::new(AxumBody::from(bytes));

        *res.status_mut() = StatusCode::OK;

        let content_type_value =
            HeaderValue::from_str(&media_type).map_err(|error| ServerFnError::ServerError {
                message: error.to_string(),
                code: 500,
                details: None,
            })?;

        res.headers_mut().insert("Content-Type", content_type_value);

        Ok(Self(res))
    }

    pub fn set_header(mut self, key: &'static str, value: HeaderValue) -> Self {
        self.0.headers_mut().insert(key, value);

        self
    }

    pub fn set_cache_control(self, value: &'static str) -> Self {
        self.set_header("Cache-Control", HeaderValue::from_static(value))
    }
}
