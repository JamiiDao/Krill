use std::{iter::repeat_with, sync::LazyLock};

use async_channel::Receiver;
use countries_iso3166::BC47LanguageInfo;
use dioxus::prelude::*;
use krill_common::{ColorSchemePreference, DynamicColorScheme, OrganizationInfo, FAVICON_DEFAULT};
use wasm_toolkit::{
    NotificationType, Notifications, WasmDocument, WasmToolkitError, WasmToolkitResult, WasmWindow,
};

use crate::{
    frontend::TAILWIND_CSS, NotificationComponent, OrgCacheOps, Route, SupportedLanguages,
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

pub(crate) static DYNAMIC_COLOR_SCHEME: GlobalSignal<DynamicColorScheme> =
    Signal::global(|| DynamicColorScheme::default());

const _: Asset = asset!("/assets/translations", AssetOptions::folder());

pub(crate) static SUPPORTED_LANGUAGES_CLIENT: GlobalSignal<SupportedLanguages> =
    Signal::global(|| SupportedLanguages::default());
pub(crate) static SELECTED_LANGUAGE: GlobalSignal<BC47LanguageInfo> =
    Signal::global(|| BC47LanguageInfo::default());

pub fn app() -> Element {
    use_effect(move || {
        spawn(async move {
            match fetch_org_info().await {
                Err(error) => {
                    let message = "Fetching organization info error. Error: `".to_string()
                        + error.to_string().as_str()
                        + "`.";

                    NOTIFICATION_MANAGER
                        .send_final(NotificationType::Failure(WasmToolkitError::Op(message)))
                        .await;
                }
                Ok(org_info_bytes) => match bitcode::decode::<OrganizationInfo>(&org_info_bytes) {
                    Ok(decoded_org_info) => {
                        if let Err(error) = OrgCacheOps::set_org_info(&decoded_org_info) {
                            NOTIFICATION_MANAGER
                                .send_final(NotificationType::Failure(error))
                                .await;
                        }
                    }
                    Err(_) => {
                        let message = "UNABLE TO DECODE ORGANIZATION INFO".to_string();
                        NOTIFICATION_MANAGER
                            .send_final(NotificationType::Failure(WasmToolkitError::Op(message)))
                            .await;
                    }
                },
            }

            load_css_variables_and_favicon().await;
            check_dark_mode().await;
            dark_mode_listener().await;
        });
    });

    rsx! {
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0",
        }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: crate::MAIN_CSS }
        document::Link { rel: "stylesheet", href: crate::FONT_STYLES }
        {crate::extra_css_styles()}
        div { class: "bg-[var(--background-color)] krill-bg-dots flex flex-col min-h-screen items-end justify-start dark:text-white light:text-black",

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
    let org_info = match OrgCacheOps::get_org_info() {
        Err(error) => {
            tracing::error!("Set OrganizationInfo to cache error: {:?}", &error);
            return;
        }
        Ok(value) => value,
    };

    if is_dark_mode {
        DYNAMIC_COLOR_SCHEME.write().set_dark_mode();
    } else {
        DYNAMIC_COLOR_SCHEME.write().set_light_mode();
    }

    match DYNAMIC_COLOR_SCHEME.read().preference() {
        ColorSchemePreference::Dark => {
            let outcome = DOCUMENT
                .read()
                .set_background_color(org_info.color_scheme.background_dark());

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
                .set_background_color(org_info.color_scheme.background_light());

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

pub(crate) async fn load_css_variables_and_favicon() {
    let org_info = match OrgCacheOps::get_org_info() {
        Err(error) => {
            tracing::error!("Set OrganizationInfo to cache error: {:?}", &error);
            return;
        }
        Ok(value) => value,
    };

    finalize_variable(
        DOCUMENT
            .read()
            .set_primary_color(org_info.color_scheme.primary_color()),
    )
    .await;

    finalize_variable(
        DOCUMENT
            .read()
            .set_secondary_color(org_info.color_scheme.secondary_color()),
    )
    .await;

    finalize_variable(
        DOCUMENT
            .read()
            .set_accent_color(org_info.color_scheme.accent_color()),
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
pub async fn fetch_org_info() -> ServerFnResult<Vec<u8>> {
    let info = crate::SERVER_ORG_INFO.get().cloned().unwrap_or_default();

    Ok(bitcode::encode(&info))
}
