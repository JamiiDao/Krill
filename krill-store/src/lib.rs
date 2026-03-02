#[cfg(feature = "server_storage")]
mod server;
#[cfg(feature = "server_storage")]
pub use server::*;

#[cfg(feature = "client_storage")]
mod client;
#[cfg(feature = "client_storage")]
pub use client::*;

#[cfg(all(feature = "server_storage", feature = "client_storage"))]
#[cfg(not(debug_assertions))]
compile_error!("Features 'client_storage' and 'server_storage' cannot be enabled together in a production build");
