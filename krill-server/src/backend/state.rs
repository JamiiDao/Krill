use std::sync::OnceLock;

use async_dup::Arc;
use async_lock::RwLock;
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use krill_common::{AuthTokenDetails, KrillError, KrillResult, ServerConfigurationState};
use krill_store::KrillStorage;

use crate::{backend::store, RouteUtils};

pub(crate) static SERVER_APP_STATE: OnceLock<Arc<RwLock<ServerConfigurationState>>> =
    OnceLock::new();

pub(crate) async fn check_app_state(request: Request, next: Next) -> impl IntoResponse {
    let path = request.uri().path();

    if path == "/" {
        return next.run(request).await;
    }

    if path.starts_with("/api/supported_languages")
        || path.starts_with("/api/fetch_org_info")
        || path.starts_with(crate::RouteUtils::LOGOUT)
        || path.starts_with(crate::RouteUtils::ERRORS)
        || path.starts_with("/logo")
    {
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
        Err(error) => return redirect_to_error(error).into_response(),
    };

    if state == ServerConfigurationState::Uninitialized {
        // allow configuration page itself
        if path == RouteUtils::CONFIGURATION || path.starts_with("/api/verification_stream") {
            return next.run(request).await;
        }

        return Redirect::to(RouteUtils::CONFIGURATION).into_response();
    }

    if state == ServerConfigurationState::LoginInitialization {
        if path.starts_with("/verify-support-mail")
            || path.starts_with("/verification-support-mail-link")
            || path.starts_with("/api/send_superuser_login_auth_link")
        {
            return next.run(request).await;
        } else {
            return Redirect::to("/verify-support-mail").into_response();
        }
    }

    // Paths that should not be run if the server state is initialized
    if path.starts_with("/api/verification_stream")
        || path.starts_with("/verification-support-mail-link")
        || path.starts_with("/api/send_superuser_login_auth_link")
        || path.starts_with("/verify-support-mail")
    {
        return Redirect::to(RouteUtils::DASHBOARD).into_response();
    }

    let fetch_cookie_outcome = fetch_cookie(request.headers()).await;

    match fetch_cookie_outcome {
        Ok(Some(_)) => {
            let path = request.uri().path();

            if path == RouteUtils::LOGIN {
                Redirect::to(RouteUtils::DASHBOARD).into_response()
            } else {
                next.run(request).await
            }
        }
        Ok(None) => {
            // if the path is LOGIN, let it through
            if path == RouteUtils::LOGIN {
                next.run(request).await
            } else {
                Redirect::to(RouteUtils::LOGIN).into_response()
            }
        }
        Err(error) => redirect_to_error(error).into_response(),
    }
}

fn redirect_to_error(error: KrillError) -> Redirect {
    let error = match error {
        KrillError::Transmit(error) => error,
        KrillError::HttpClient(error) => error,
        KrillError::HttpResponse(error) => error,
        _ => {
            tracing::error!("{:?}", error);

            "Encountered fatal error".to_string()
        }
    };

    let error_route = crate::RouteUtils::ERRORS.to_string() + "/" + error.as_str();

    Redirect::to(&error_route)
}

pub(crate) async fn load_app_state(store: &KrillStorage) -> KrillResult<ServerConfigurationState> {
    let app_state = store.get_app_state().await?;

    SERVER_APP_STATE
        .set(Arc::new(RwLock::new(app_state)))
        .or(Err(KrillError::UnableToSetAppState))?;

    Ok(app_state)
}

pub async fn server_state() -> KrillResult<ServerConfigurationState> {
    let state = SERVER_APP_STATE
        .get()
        .ok_or(KrillError::AppStateMachineNotInitialized)?;

    Ok(*state.read().await)
}

pub(crate) async fn fetch_cookie(
    headers: &HeaderMap,
) -> KrillResult<Option<[u8; AuthTokenDetails::AUTH_TOKEN_LEN]>> {
    let jar = CookieJar::from_headers(headers);

    let session_cookie =
        if let Some(cookie) = jar.get(AuthTokenDetails::COOKIE_AUTH_TOKEN_IDENTIFIER) {
            cookie.value()
        } else {
            return Ok(Option::None);
        };

    let cookie_hash: [u8; AuthTokenDetails::AUTH_TOKEN_LEN] =
        if let Ok(hash) = AuthTokenDetails::decode_token(session_cookie.trim()) {
            hash
        } else {
            return Ok(None);
        };

    let storage = store()?;

    Ok(storage
        .get_auth_token(cookie_hash)
        .await?
        .map(|_| cookie_hash))
}
