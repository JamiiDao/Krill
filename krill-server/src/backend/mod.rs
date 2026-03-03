mod home;
pub use home::*;

#[cfg(feature = "server")]
mod init_store;
#[cfg(feature = "server")]
pub use init_store::*;

#[cfg(feature = "server")]
mod state;
#[cfg(feature = "server")]
pub use state::*;

#[cfg(feature = "server")]
mod supervisor_demo;
#[cfg(feature = "server")]
pub use supervisor_demo::*;

#[cfg(feature = "server")]
pub fn server_outcome_response<T: bitcode::Encode + bitcode::Decode<'static>>(
    outcome: krill_common::KrillResult<T>,
) -> dioxus::fullstack::axum_core::response::Response {
    let outcome = krill_common::ServerOutcome::<T>::encode(outcome);

    dioxus::fullstack::axum_core::response::Response::new(outcome.into())
}
