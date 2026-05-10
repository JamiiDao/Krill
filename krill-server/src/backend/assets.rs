use dioxus::{fullstack::response::Response, prelude::*};

#[get("/logo")]
pub async fn send_logo() -> ServerFnResult<Response> {
    use crate::backend::MediaTypeHttp;

    let logo = crate::backend::SERVER_ORG_INFO
        .get()
        .map(|info| info.logo.clone())
        .unwrap_or_default();

    MediaTypeHttp::new_one_day_cache(logo)
}
