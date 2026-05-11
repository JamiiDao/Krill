use dioxus::{prelude::*, Result as DioxusResult};

#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};

#[get("/dashboard-data",  header: TypedHeader<Cookie>)]
pub async fn dashboard_data() -> DioxusResult<()> {
    // use krill_common::AuthTokenDetails;

    // let cookie = header
    //     .get(AuthTokenDetails::COOKIE_AUTH_TOKEN_IDENTIFIER)
    //     .or_unauthorized("Missing auth-demo cookie")?
    //     .eq(THIS_SESSION_ID.to_string().as_str())
    //     .or_unauthorized("Invalid auth-demo cookie")?;

    // tracing::info!("HEADER GROUP: {:?}", cookie);

    Ok(())
}
