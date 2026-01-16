#[cfg(feature = "server")]
use dioxus::server::axum;

mod frontend;
use frontend::*;

mod backend;
use backend::*;

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);

    #[cfg(feature = "server")]
    crate::init_server_statics().unwrap();

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        Ok(dioxus::server::router(app).layer(axum::middleware::from_fn(check_app_state)))
    });
}

// #[cfg(feature = "server")]
// async fn setup_language(request: Request) -> Response {
//     Response::builder()
//         .status(200)
//         .body("Select language".into())
//         .unwrap()
// }

// #[cfg(feature = "server")]
// async fn setup_logos(request: Request) -> Response {
//     Response::builder()
//         .status(200)
//         .body("Upload logo".into())
//         .unwrap()
// }
