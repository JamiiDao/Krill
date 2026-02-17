use std::sync::OnceLock;

use async_dup::Arc;
use async_lock::RwLock;
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use krill_common::{KrillError, KrillResult};
use krill_store::{KrillStorage, ServerCookie};

use crate::RouteUtils;

pub(crate) static SERVER_APP_STATE: OnceLock<Arc<RwLock<bool>>> = OnceLock::new();

pub(crate) async fn check_app_state(request: Request, next: Next) -> impl IntoResponse {
    let path = request.uri().path();

    // Load paths like login without auth
    if path.starts_with("/login") || path.starts_with("/logout") {
        return next.run(request).await;
    }

    // Load assets without auth
    if path.starts_with("/assets")
        || path.starts_with("/pkg")
        || path.ends_with(".js")
        || path.ends_with(".wasm")
        || path.ends_with(".css")
        || path.ends_with(".ico")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".svg")
    {
        return next.run(request).await;
    }

    let state = match server_state().await {
        Ok(state) => state,
        Err(_) => return Redirect::temporary(RouteUtils::APP_ERROR).into_response(),
    };

    if !state {
        // allow configuration page itself
        if path == RouteUtils::CONFIGURATION {
            return next.run(request).await;
        }

        return Redirect::temporary(RouteUtils::CONFIGURATION).into_response();
    }

    // App configured
    match fetch_cookie(request.headers()).await {
        Ok(Some(_)) => next.run(request).await,
        Ok(None) => Redirect::temporary(RouteUtils::LOGIN).into_response(),
        Err(_) => Redirect::temporary(RouteUtils::APP_ERROR).into_response(),
    }
}

pub(crate) async fn load_app_state(store: &KrillStorage) -> KrillResult<bool> {
    let app_state = store.get_app_state().await?;

    SERVER_APP_STATE
        .set(Arc::new(RwLock::new(app_state)))
        .or(Err(KrillError::UnableToSetAppState))?;

    Ok(app_state)
}

pub async fn server_state() -> KrillResult<bool> {
    let state = SERVER_APP_STATE
        .get()
        .ok_or(KrillError::AppStateMachineNotInitialized)?;

    Ok(*state.read().await)
}

pub(crate) async fn fetch_cookie(headers: &HeaderMap) -> KrillResult<Option<ServerCookie>> {
    use base64ct::{Base64, Encoding};

    let jar = CookieJar::from_headers(headers);

    let session_cookie = if let Some(cookie) = jar.get(ServerCookie::IDENTIFIER) {
        cookie.value()
    } else {
        return Ok(Option::None);
    };

    let decoded = match Base64::decode_vec(session_cookie).ok() {
        None => return Ok(Option::None),
        Some(value) => value,
    };

    let as_cookie = match bitcode::decode::<ServerCookie>(&decoded).ok() {
        None => return Ok(Option::None),
        Some(value) => value,
    };

    let hash = blake3::Hash::from_bytes(as_cookie.hash);

    let rehashed = ServerCookie::hash(&as_cookie.data);

    let outcome = if hash != rehashed {
        Option::None
    } else {
        Some(as_cookie)
    };

    Ok(outcome)
}
