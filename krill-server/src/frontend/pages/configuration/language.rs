use countries_iso3166::BC47LanguageInfo;
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_toolkit::{NotificationType, WasmToolkitError};

use crate::{
    Loader, ProgressStateToUiRecord, SupportedLanguages, Translations, NOTIFICATION_MANAGER,
    SELECTED_LANGUAGE, SUPPORTED_LANGUAGES_CLIENT,
};

#[component]
pub fn LanguageView() -> Element {
    let state_data = consume_context::<Signal<ProgressStateToUiRecord>>();
    let translations = consume_context::<Signal<Translations>>();

    let fetched_langs = use_server_future(|| async move {
        let fetched = crate::supported_languages()
            .await
            .map_err(|error| error.to_string());

        if let Err(error) = fetched.as_ref() {
            NOTIFICATION_MANAGER
                .send_final(NotificationType::Failure(WasmToolkitError::Op(
                    error.to_string(),
                )))
                .await;
        }

        let parsed = SupportedLanguages::parse(fetched?);

        match parsed {
            Err(error) => {
                NOTIFICATION_MANAGER
                    .send_final(NotificationType::Failure(WasmToolkitError::Op(
                        error.to_string(),
                    )))
                    .await;

                Err(error.to_string())
            }
            Ok(value) => {
                *SUPPORTED_LANGUAGES_CLIENT.write() = value.clone();

                Ok(value)
            }
        }
    });

    use_effect(move || {
        *SELECTED_LANGUAGE.write() = state_data.read().language();
    });

    #[rustfmt::skip]
    rsx! {
        div { class: "transition duration-1000 ease-in-out flex flex-col h-full w-full items-center justify-center",
            match fetched_langs {
                Err(_) => rsx!{
                    div {class:"flex w-[80%] items-center justify-center",
                        Loader {  }
                    }
                },
                Ok(resource) => {                   
                    rsx! {
                        div { class: "flex flex-col w-full h-full p-5 items-center justify-center",
                            div { class: "flex text-md lg:text-2xl font-[markoonefont] text-[var(--secondary-color)] mb-1", 
                                
                                {translations.read().translate("select_default_lang")}
                            }

                            div { class:"flex mb-5 w-full" }

                            match resource.read().as_ref() {
                                Some(Ok(value)) => {
                                    create_list(value,  state_data,translations)
                                },
                                Some(Err(error)) => {
                                    rsx!{"Fetching languages error {error}"}
                                },
                                None => {
                                    rsx!{Loader {}}
                                }
                            }
                        
                        }
                    }
                }
            }
        }
    }
}

fn create_list(
    supported_langs: &SupportedLanguages,
    mut state_data: Signal<ProgressStateToUiRecord>,
    mut translations: Signal<Translations>,
) -> Element {
    rsx! {
        ul { class: "overflow-y-auto flex flex-col w-[90%] md:w-[70%] lg:w-[50%] items-start ",

            {
                supported_langs
                    .get_langs()
                    .iter()
                    .map(|lang| {
                        let lang = *lang;
                        rsx! {
                            li {
                                id: lang.code().to_string(),
                                onclick: move |_| {
                                    if let Err(error) = state_data.write().set_language(lang) {
                                        spawn(async move {
                                            NOTIFICATION_MANAGER
                                                .send_final(
                                                    NotificationType::Failure(
                                                        WasmToolkitError::Op(error.to_string()),
                                                    ),
                                                )
                                                .await;
                                        });
                                    }

                                    if let Err(error) = state_data.write().transition() {
                                        spawn(async move {
                                            NOTIFICATION_MANAGER
                                                .send_final(
                                                    NotificationType::Failure(
                                                        WasmToolkitError::Op(error.to_string()),
                                                    ),
                                                )
                                                .await;
                                        });
                                    }

                                    *SELECTED_LANGUAGE.write() = lang;

                                    spawn_local(async move {
                                        match Translations::get_translation(
                                                "configuration",
                                                SELECTED_LANGUAGE.read().code(),
                                            )
                                            .await
                                        {
                                            Ok(fetched_translations) => {
                                                translations.set(fetched_translations);
                                            }
                                            Err(error) => {
                                                NOTIFICATION_MANAGER
                                                    .send_final(
                                                        wasm_toolkit::NotificationType::Failure(
                                                            wasm_toolkit::WasmToolkitError::Op(error.to_string()),
                                                        ),
                                                    )
                                                    .await
                                            }
                                        };
                                    });

                                },
                                class: "flex cursor-pointer w-full p-2 items-start justify-center text-sm md:text-lg text-start",
                                class: if *SELECTED_LANGUAGE.read() == lang { "hover:bg-transparent rounded-full" } else { "hover:bg-[var(--secondary-color)] rounded-full" },
                                if *SELECTED_LANGUAGE.read() == lang {
                                    svg {
                                        "aria_hidden": "true",
                                        class: "w-4 h-4 lg:w-6 lg:h-6 text-fg-success me-1.5 shrink-0",
                                        fill: "none",
                                        height: "24",
                                        view_box: "0 0 24 24",
                                        width: "24",
                                        xmlns: "http://www.w3.org/2000/svg",
                                        path {
                                            d: "M8.5 11.5 11 14l4-4m6 2a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
                                            stroke: "var(--primary-color)",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                        }
                                    }
                                } else {
                                    div { class: "w-4 h-4 lg:w-6 lg:h-6 me-1.5 shrink-0" }
                                }
                                span {
                                    class: "flex",
                                    class: if *SELECTED_LANGUAGE.read() == lang { "text-[var(--primary-color)]" },
                                    if lang == BC47LanguageInfo::EN_US {
                                        "{lang.native()} "
                                    } else {
                                        "{lang.native()} - {lang.english()}"
                                    }
                                }
                            }
                        }
                    })
            }
        }
    }
}
