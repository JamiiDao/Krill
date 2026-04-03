use std::{iter::repeat_with, sync::LazyLock};

use async_channel::Receiver;
use countries_iso3166::BC47LanguageInfo;
use dioxus::prelude::*;
use krill_common::{ColorScheme, ColorSchemePreference, DynamicColorScheme};
use wasm_toolkit::{
    NotificationType, Notifications, WasmDocument, WasmToolkitError, WasmToolkitResult, WasmWindow,
};

use crate::{
    frontend::{FAVICON, TAILWIND_CSS},
    NotificationComponent, Route, SupportedLanguages,
};

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

pub(crate) static CLIENT_COLOR_SCHEME: GlobalSignal<ColorScheme> =
    Signal::global(|| ColorScheme::default());

pub(crate) static DYNAMIC_COLOR_SCHEME: GlobalSignal<DynamicColorScheme> =
    Signal::global(|| DynamicColorScheme::default());

const _: Asset = asset!("/assets/translations", AssetOptions::folder());

pub(crate) static SUPPORTED_LANGUAGES_CLIENT: GlobalSignal<SupportedLanguages> =
    Signal::global(|| SupportedLanguages::default());
pub(crate) static SELECTED_LANGUAGE: GlobalSignal<BC47LanguageInfo> =
    Signal::global(|| BC47LanguageInfo::default());

pub fn app() -> Element {
    let color_scheme = use_server_future(move || async move {
        let outcome = fetch_color_scheme().await;

        match outcome {
            Err(error) => {
                let message = "Fetching brand colors error. Error: `".to_string()
                    + error.to_string().as_str()
                    + "`.";

                NOTIFICATION_MANAGER
                    .send_final(NotificationType::Failure(WasmToolkitError::Op(message)))
                    .await;
            }
            Ok(color_scheme_ok) => match bitcode::decode::<ColorScheme>(&color_scheme_ok) {
                Ok(decoded_color_scheme) => {
                    *CLIENT_COLOR_SCHEME.write() = decoded_color_scheme;
                }
                Err(_) => {
                    let message = "UNABLE TO DECODE BRAND COLORS".to_string();
                    NOTIFICATION_MANAGER
                        .send_final(NotificationType::Failure(WasmToolkitError::Op(message)))
                        .await;
                }
            },
        }
    });

    use_effect(move || {
        spawn(async move {
            load_css_variables().await;
            check_dark_mode().await;
            dark_mode_listener().await;
        });
    });

    rsx! {
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0",
        }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: crate::MAIN_CSS }
        document::Link { rel: "stylesheet", href: crate::FONT_STYLES }
        {crate::extra_css_styles()}
        div { class: "bg-[var(--background-color)] flex flex-col min-h-screen items-end justify-start dark:text-white light:text-black",

            {
                if let Err(error) = color_scheme {
                    {
                        tracing::error!(
                            "Fetching Color Scheme (Brand Colors). Error: `{}`", error
                            .to_string()
                        );
                    }
                    rsx! {}
                } else {
                    rsx! {}
                }
            }
            NotificationComponent {}
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

    match DYNAMIC_COLOR_SCHEME.read().preference() {
        ColorSchemePreference::Dark => {
            let outcome = DOCUMENT
                .read()
                .set_background_color(CLIENT_COLOR_SCHEME.read().background_dark());

            if let Err(error) = outcome {
                NOTIFICATION_MANAGER
                    .send_final(NotificationType::Failure(WasmToolkitError::Op(
                        error.to_string(),
                    )))
                    .await;
            }
        }
        ColorSchemePreference::Light => {
            let outcome = DOCUMENT
                .read()
                .set_background_color(CLIENT_COLOR_SCHEME.read().background_light());

            if let Err(error) = outcome {
                NOTIFICATION_MANAGER
                    .send_final(NotificationType::Failure(WasmToolkitError::Op(
                        error.to_string(),
                    )))
                    .await;
            }
        }
        ColorSchemePreference::PitchBlack => {
            if let Err(error) = DOCUMENT.read().set_background_color_pitch_black() {
                NOTIFICATION_MANAGER
                    .send_final(NotificationType::Failure(WasmToolkitError::Op(
                        error.to_string(),
                    )))
                    .await;
            }
        }
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
    finalize_variable(
        DOCUMENT
            .read()
            .set_primary_color(CLIENT_COLOR_SCHEME.read().primary_color()),
    )
    .await;

    finalize_variable(
        DOCUMENT
            .read()
            .set_secondary_color(CLIENT_COLOR_SCHEME.read().secondary_color()),
    )
    .await;

    finalize_variable(
        DOCUMENT
            .read()
            .set_accent_color(CLIENT_COLOR_SCHEME.read().accent_color()),
    )
    .await;
}

async fn finalize_variable(outcome: WasmToolkitResult<()>) {
    if let Err(error) = outcome {
        NOTIFICATION_MANAGER
            .send_final(NotificationType::Failure(error))
            .await
    }
}

#[server]
async fn fetch_color_scheme() -> ServerFnResult<Vec<u8>> {
    crate::SERVER_ORG_INFO
        .get()
        .cloned()
        .ok_or(ServerFnError::ServerError {
            message: "Unable to fetch brand colors".to_string(),
            code: 500,
            details: None,
        })
        .map(|info| bitcode::encode(&info))
}
