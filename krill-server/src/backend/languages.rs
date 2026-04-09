use dioxus::prelude::*;

#[server]
pub async fn supported_languages() -> ServerFnResult<Vec<String>> {
    Ok(crate::default_langs())
}
