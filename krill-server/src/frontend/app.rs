use std::{
    cell::RefCell,
    iter::repeat_with,
    rc::Rc,
    sync::{LazyLock, OnceLock},
};

use dioxus::prelude::*;
use krill_common::{ColorScheme, ColorSchemePreference, DynamicColorScheme};
use wasm_toolkit::{
    NotificationType, Notifications, WasmDocument, WasmToolkitError, WasmToolkitResult, WasmWindow,
};

use crate::{
    frontend::{FAVICON, TAILWIND_CSS},
    NotificationComponent, Route,
};

#[allow(clippy::redundant_closure)]
pub(crate) static NOTIFICATION_MANAGER: LazyLock<Notifications> =
    LazyLock::new(|| Notifications::init());

pub(crate) static WINDOW: GlobalSignal<WasmWindow> =
    Signal::global(|| WasmWindow::new().expect("Unable to get the browser window"));

pub(crate) static DOCUMENT: GlobalSignal<WasmDocument> = Signal::global(|| {
    let document = WINDOW
        .read()
        .document()
        .expect("Unable to get the browser window")
        .inner()
        .clone();

    let document = WasmDocument::new(document);
    document
        .set_background_color_pitch_black()
        .expect("Unable to set pitch black default");

    document
});

#[allow(clippy::redundant_closure)]
pub(crate) static CLIENT_COLOR_SCHEME: OnceLock<ColorScheme> = OnceLock::new();

#[allow(clippy::redundant_closure)]
pub(crate) static DYNAMIC_COLOR_SCHEME: GlobalSignal<DynamicColorScheme> =
    Signal::global(|| DynamicColorScheme::default());

pub fn app() -> Element {
    use_effect(move || {
        spawn(load_css_variables());
        spawn(check_dark_mode());
    });

    let color_scheme = use_server_future(|| fetch_color_scheme());

    spawn(async move {
        // let timeout = gloo_timers::callback::Interval::new(1000, move || {
        //     wasm_bindgen_futures::spawn_local(async move {
        //         let random_notification: String =
        //             repeat_with(fastrand::alphanumeric).take(10).collect();

        //         NOTIFICATION_MANAGER
        //             .send_final(NotificationType::Success(random_notification))
        //             .await;
        //     });
        // });

        // timeout.forget();

        wasm_bindgen_futures::spawn_local(async move {
            let random_notification: String =
                repeat_with(fastrand::alphanumeric).take(10).collect();

            NOTIFICATION_MANAGER
                .send_final(NotificationType::Success(random_notification))
                .await;
        });
    });

    spawn(dark_mode_listener());

    rsx! {
        document::Meta {name: "viewport", content: "width=device-width, initial-scale=1.0"}
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Style { r#"
                body {{
                    background-color: #000;
                }}
            "#

        }
        div {class:"bg-[var(--background-color)] flex flex-col min-h-screen items-end justify-start",
            {
                match color_scheme {
                    Err(_) => {
                        rsx!{"UNABLE TO RENDER BRAND COLORS"}
                    },
                    Ok(color_scheme_ok) => {
                        match bitcode::decode::<ColorScheme>(&color_scheme_ok.value().unwrap().unwrap()){
                            Ok(decoded_color_scheme) => {
                                    CLIENT_COLOR_SCHEME.set(decoded_color_scheme).err();
                                rsx! {"The resource is ready!"}
                            },
                            Err(_) => {
                                rsx!{"UNABLE TO DECODE BRAND COLORS"}
                            }
                        }
                    }
                }
            }

            NotificationComponent{}
            Router::<Route> {}
        }
    }
}

async fn check_dark_mode() {
    match WINDOW.read().is_dark_mode() {
        Ok(is_dark_mode) => {
            match_bg_scheme(is_dark_mode).await;
        }
        Err(error) => {
            NOTIFICATION_MANAGER
                .send_final(NotificationType::Failure(error))
                .await
        }
    }
}

async fn match_bg_scheme(is_dark_mode: bool) {
    if is_dark_mode {
        DYNAMIC_COLOR_SCHEME.write().set_dark_mode();
    } else {
        DYNAMIC_COLOR_SCHEME.write().set_light_mode();
    }

    let outcome = match DYNAMIC_COLOR_SCHEME.read().preference() {
        ColorSchemePreference::Dark => DOCUMENT
            .read()
            .set_background_color(CLIENT_COLOR_SCHEME.get().unwrap().background_dark()),
        ColorSchemePreference::Light => DOCUMENT
            .read()
            .set_background_color(CLIENT_COLOR_SCHEME.get().unwrap().background_light()),

        ColorSchemePreference::PitchBlack => DOCUMENT.read().set_background_color_pitch_black(),
    };

    if let Some(error) = outcome.err() {
        NOTIFICATION_MANAGER
            .send_final(NotificationType::Failure(error))
            .await
    }
}

async fn dark_mode_listener() {
    match WINDOW.read().watch_dark_mode().await {
        Err(error) => {
            NOTIFICATION_MANAGER
                .send_final(NotificationType::Failure(error))
                .await
        }
        Ok(id_dark_mode_listener) => {
            while let Ok(is_dark_mode) = id_dark_mode_listener.recv().await {
                match_bg_scheme(is_dark_mode).await;
            }
        }
    }
}

async fn load_css_variables() {
    let brand_colors = match CLIENT_COLOR_SCHEME.get() {
        Some(value) => value,
        None => {
            NOTIFICATION_MANAGER
                .send_final(NotificationType::Failure(WasmToolkitError::Op(
                    "BRAND_COLORS NOT SET".to_string(),
                )))
                .await;

            return;
        }
    };

    finalize_variable(
        DOCUMENT
            .read()
            .set_primary_color(brand_colors.primary_color()),
    )
    .await;

    finalize_variable(
        DOCUMENT
            .read()
            .set_secondary_color(brand_colors.secondary_color()),
    )
    .await;

    finalize_variable(
        DOCUMENT
            .read()
            .set_accent_color(brand_colors.accent_color()),
    )
    .await;
}

async fn finalize_variable(outcome: WasmToolkitResult<()>) {
    if let Some(error) = outcome.err() {
        NOTIFICATION_MANAGER
            .send_final(NotificationType::Failure(error))
            .await
    }
}

#[server]
async fn fetch_color_scheme() -> ServerFnResult<Vec<u8>> {
    Ok(crate::SERVER_COLOR_SCHEME.get().unwrap().clone())
}
