use std::{cell::RefCell, iter::repeat_with, rc::Rc, sync::LazyLock};

use dioxus::prelude::*;
use krill_common::{ColorScheme, ColorSchemePreference};
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
pub(crate) static COLOR_SCHEME: GlobalSignal<Rc<RefCell<ColorScheme>>> =
    Signal::global(|| Rc::new(RefCell::new(ColorScheme::new())));

pub fn app() -> Element {
    use_effect(move || {
        spawn(check_dark_mode());
    });

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
        COLOR_SCHEME.write().borrow_mut().set_dark_mode();
    } else {
        COLOR_SCHEME.write().borrow_mut().set_light_mode();
    }

    let outcome = match COLOR_SCHEME.read().borrow().preference() {
        ColorSchemePreference::Dark => DOCUMENT
            .read()
            .set_background_color(COLOR_SCHEME.read().borrow().background_dark()),
        ColorSchemePreference::Light => DOCUMENT
            .read()
            .set_background_color(COLOR_SCHEME.read().borrow().background_light()),

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
