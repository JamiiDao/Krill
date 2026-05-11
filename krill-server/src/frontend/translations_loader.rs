use dioxus::prelude::*;

use crate::{Translations, NOTIFICATION_MANAGER, SELECTED_LANGUAGE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationsMemInfo {
    pub loading: bool,
    pub translations: Translations,
}

impl TranslationsMemInfo {
    pub fn new() -> Self {
        Self {
            loading: true,
            translations: Translations::default(),
        }
    }

    pub async fn fetch(page: &str, mut mem_store: Signal<Self>) {
        match Translations::get_translation(page, SELECTED_LANGUAGE.read().code()).await {
            Ok(translations) => {
                mem_store.set(TranslationsMemInfo {
                    translations,
                    loading: false,
                });
            }
            Err(error) => {
                NOTIFICATION_MANAGER
                    .send_final(wasm_toolkit::NotificationType::Failure(
                        wasm_toolkit::WasmToolkitError::Op(error.to_string()),
                    ))
                    .await
            }
        };
    }
}
