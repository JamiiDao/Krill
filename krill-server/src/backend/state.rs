#[cfg(feature = "server")]
use {
    axum::{
        extract::Request,
        middleware::Next,
        response::{Html, IntoResponse},
    },
    dioxus::prelude::*,
    krill_common::AppStateMachine,
};

#[cfg(feature = "server")]
pub async fn check_app_state(request: Request, next: Next) -> impl IntoResponse {
    use axum_extra::extract::CookieJar;

    use crate::backend::store;

    // Create a CookieJar from the incoming request headers
    let jar = CookieJar::from_headers(request.headers());

    // Example: check for a "session" cookie
    if let Some(cookie) = jar.get("session") {
        let session_value = cookie.value();
        // TODO: validate session / perform authorization
        println!("Session cookie: {}", session_value);
    } else {
        // No session cookie → reject
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    next.run(request).await

    // match store() {
    //     Err(error) => app_eror_response(error),
    //     Ok(store) => {
    //          store.get_app_state().await
    //     },
    // }
}

// pub async fn app_eror_response() -> impl IntoResponse {
//     (StatusCode::OK, Html ::from("ok"))
// }
