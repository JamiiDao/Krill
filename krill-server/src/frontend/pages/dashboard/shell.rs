use dioxus::prelude::*;
use wasm_toolkit::WasmToolkitError;

use crate::{Header, LoadingLanguageTranslation, TranslationsMemInfo, NOTIFICATION_MANAGER};

#[component]
pub fn Dashboard() -> Element {
    let dashboard_details = use_server_future(|| async move {
        match crate::dashboard_data().await {
            Ok(value) => value,
            Err(error) => {
                NOTIFICATION_MANAGER
                    .send_final(wasm_toolkit::NotificationType::Failure(
                        WasmToolkitError::JsError {
                            name: "Dashboard details".to_string(),
                            message: error.to_string(),
                        },
                    ))
                    .await;
            }
        }
    })?;

    use_context_provider(|| Signal::new(TranslationsMemInfo::new()));

    let mut loading_langs = use_signal(|| true);

    let translations_info = consume_context::<Signal<TranslationsMemInfo>>();

    use_effect(move || {
        spawn(async move {
            TranslationsMemInfo::fetch("dashboard", translations_info).await;

            loading_langs.set(false);
        });
    });

    if *loading_langs.read() {
        rsx! {
            LoadingLanguageTranslation {}
        }
    } else {
        match dashboard_details.read().as_ref() {
            Some(()) => rsx! {
                div { class: "h-screen w-full flex flex-col justify-start items-center krill-bg-dots ",
                    Header {}
                }
            },
            None => {
                rsx! {}
            }
        }
    }
}
