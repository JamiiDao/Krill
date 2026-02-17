mod frontend;
pub(crate) use frontend::*;

mod backend;
use backend::*;

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);

    #[cfg(feature = "server")]
    crate::init_server_statics().unwrap();

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use axum::{extract::Request, middleware::Next};

        Ok(dioxus::server::router(app).layer(axum::middleware::from_fn(
            |request: Request, next: Next| async move { check_app_state(request, next).await },
        )))
    });
}
