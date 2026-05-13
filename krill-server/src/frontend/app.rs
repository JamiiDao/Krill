use std::{iter::repeat_with, sync::LazyLock};

use async_channel::bounded;
use countries_iso3166::BC47LanguageInfo;
use dioxus::prelude::*;
use krill_common::{ColorSchemePreference, DynamicColorScheme, OrganizationInfo};
use wasm_toolkit::{
    BrowserMeasurements, NotificationType, Notifications, WasmDocument, WasmToolkitError,
    WasmToolkitResult, WasmWindow,
};

use crate::{
    frontend::TAILWIND_CSS, ErrorComponent, ErrorUtil, Loader, NotificationComponent, OrgCacheOps,
    Route, SupportedLanguages,
};

pub const KRILL_GLASS: &str =
    "krill-bg-surface-container krill-backdrop-blur-glass krill-shadow-glass";

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
pub(crate) static BROWSER_MEASUREMENTS: GlobalSignal<BrowserMeasurements> =
    Signal::global(|| BrowserMeasurements::default());

pub fn app() -> Element {
    let (sender, receiver) = bounded::<Result<BrowserMeasurements, Vec<WasmToolkitError>>>(5);

    spawn(async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                Ok(value) => {
                    *BROWSER_MEASUREMENTS.write() = value;
                }
                Err(errors) => {
                    for error in errors {
                        ErrorUtil::send_final_wasm_toolkit(error).await;
                    }
                }
            }
        }
    });

    let load_org_info = use_server_future(|| fetch_org_info())?;
    use_context_provider(|| Signal::new(OrganizationInfo::default()));

    match load_org_info.result() {
        Some(Ok(value_result)) => {
            let mut error_to_ui = use_signal(|| Option::<String>::default());

            use_future(move || {
                let sender_cloned = sender.clone();
                async move {
                    spawn(async move {
                        load_measurements().await;

                        if let Err(error) =
                            WINDOW.read().browser_measurements_listener(sender_cloned)
                        {
                            error_to_ui.write().replace(error.to_string());
                        }

                        if let Ok(decoded_org_info) = bitcode::decode(&value_result.read()) {
                            if let Err(error) = OrgCacheOps::set_org_info(&decoded_org_info) {
                                ErrorUtil::send_final_wasm_toolkit(error).await;
                            } else {
                                let mut org_info = consume_context::<Signal<OrganizationInfo>>();
                                org_info.set(decoded_org_info);
                            }
                        } else {
                            error_to_ui
                                .write()
                                .replace("Unable to decode `OrganizationInfo`!".to_string());
                        }

                        bind_org_info_to_page().await;
                        check_dark_mode().await;
                        dark_mode_listener().await;
                    });
                }
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
                    match error_to_ui.as_ref() {
                        Some(value) => {
                            rsx! {
                                ErrorComponent { stringyfied: value.to_string() }
                            }
                        }
                        None => {
                            rsx! {
                                NotificationComponent {}
                                Router::<Route> {}
                            }
                        }
                    }
                }
            }
        }
        Some(Err(error)) => rsx! {
            ErrorComponent { stringyfied: error.to_string() }
        },
        None => rsx! {
            Loader {}
        },
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

async fn load_measurements() {
    match WINDOW.read().get_browser_measurements() {
        Err(error) => NOTIFICATION_MANAGER.send_final_error(error).await,
        Ok(value) => *BROWSER_MEASUREMENTS.write() = value,
    }
}

pub(crate) async fn bind_org_info_to_page() {
    let org_info = match OrgCacheOps::get_org_info() {
        Err(error) => {
            tracing::error!("Set OrganizationInfo to cache error: {:?}", &error);
            return;
        }
        Ok(value) => value,
    };

    DOCUMENT.read().set_page_title(&org_info.name);

    finalize_variable(DOCUMENT.read().set_favicon(&org_info.favicon)).await;

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
pub async fn fetch_org_info() -> dioxus::Result<Vec<u8>> {
    let info = crate::SERVER_ORG_INFO.get().cloned().unwrap_or_default();

    Ok(bitcode::encode(&info))
}
