use dioxus::prelude::*;
use wasm_toolkit::{NotificationType, WasmToolkitError};

use crate::NOTIFICATION_MANAGER;

#[component]
pub fn LanguageView() -> Element {
    let fetched_langs = use_server_future(|| async move {
        let fetched = crate::langs_with_translations().await;

        if let Err(error) = fetched.as_ref() {
            NOTIFICATION_MANAGER
                .send_final(NotificationType::Failure(WasmToolkitError::Op(
                    error.to_string(),
                )))
                .await;
        }

        fetched
    });

    rsx! {
        div { class:"[&>li:nth-child(odd)]:bg-gray-100 [&>li:nth-child(even)]:bg-white
            w-full",

            {match fetched_langs {
                Ok(lang_codes_result) => {
                    if lang_codes_result.finished() {
                        tracing::info!("LANGS FETCH SUCCESS: {:?}",lang_codes_result.read())
                    }else {
                        tracing::info!("LANGS FETCH Loading....")
                    }
                },
                Err(error) => {

                }
            }}
         }
    }
}
