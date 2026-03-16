use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[server]
pub async fn supported_languages() -> ServerFnResult<Vec<String>> {
    Ok(crate::SUPPORTED_LANGUAGES
        .get()
        .ok_or(ServerFnError::ServerError {
            message: "SUPPORTED LANGUAGES MAY NOT BE INITIALIZED".to_string(),
            code: 400,
            details: None,
        })?
        .clone())
}
