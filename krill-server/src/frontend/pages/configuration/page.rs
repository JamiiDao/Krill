use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_toolkit::WasmToolkitResult;

use crate::{LanguageView, Loader, Translations, NOTIFICATION_MANAGER, WINDOW};

#[component]
pub fn Configuration() -> Element {
    let shield_logo: Asset = asset!("/assets/krill-shield-logo.svg");

    let mut state_data = use_signal(|| Option::<ProgressStateToUiRecord>::default());

    let mut translations = use_signal(|| Translations::default());

    use_effect(move || {
        spawn(async move {
            match ProgressStateToUiRecord::from_session_storage() {
                Ok(state_may_exist) => {
                    state_data.set(Some(state_may_exist));
                }
                Err(error) => {
                    NOTIFICATION_MANAGER
                        .send_final(wasm_toolkit::NotificationType::Failure(error))
                        .await
                }
            }
        });
    });

    use_effect(move || {
        spawn(async move {
            match Translations::get_translation("configuration").await {
                Ok(fetched_translations) => {
                    translations.set(fetched_translations);
                }
                Err(error) => {
                    NOTIFICATION_MANAGER
                        .send_final(wasm_toolkit::NotificationType::Failure(
                            wasm_toolkit::WasmToolkitError::Op(error.to_string()),
                        ))
                        .await
                }
            };
        });
    });

    rsx! {
        div { class:"flex flex-col min-h-screen w-[90dvw] self-center items-center justify-center text-center grid-3-children",
            div{class:"flex flex-col w-full justify-end items-center h-full p-1",
                img {class:"flex w-[35%] md:w-[20%] lg:w-[15%]", src:"{shield_logo}", alt:"Krill Shield Logo" }
            }

            if let Some(page_state_data) = state_data.read().as_ref() {
                div{class:"flex flex-col w-full justify-center items-center h-full p-2",
                    {progress_info(page_state_data,&translations.read())}
                }

                div{class:"flex flex-col w-full justify-center items-center h-full",
                    match page_state_data.progress_state {
                        ConfigurationProgress::Language => rsx!{ LanguageView {} },
                        ConfigurationProgress::OrgInfo => rsx!{ OrgInfo{} },
                        ConfigurationProgress::SmtpSecure => rsx!{SmtpSecure{}},
                        ConfigurationProgress::Passcode => rsx!{Passcode {}},
                    }
                }
            }else {
                Loader {}
            }
        }
    }
}

fn progress_info(
    progress_state_data: &ProgressStateToUiRecord,
    translations: &Translations,
) -> Element {
    rsx! {
        div {class:"flex w-full grid-column-7",
            {progress_info_item(&progress_state_data.language_state, translations)}
            ProgressInfoItemBar{}
            {progress_info_item(&progress_state_data.org_info_state, translations)}
            ProgressInfoItemBar{}
            {progress_info_item(&progress_state_data.smtp_state, translations)}
            ProgressInfoItemBar{}
            {progress_info_item(&progress_state_data.passcode_state, translations)}
        }
    }
}

fn progress_info_item(progress_data: &ProgressStateData, translations: &Translations) -> Element {
    rsx! {
        div {class:"item w-full flex rounded-full p-5 items-center justify-center",

            {match progress_data.state {
                ProgressStateToUi::Inactive => rsx!{
                    span {class:"flex mr-1 max-w-[20px] min-w-[15px] w-[20%] rounded-full p-[1px]",
                        StrokeFillSvg {  }
                    }
                },
                ProgressStateToUi::Active => rsx!{
                    span {class:"flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                        FillSvg {  }
                    }
                },
                ProgressStateToUi::Processed => rsx!{
                    span {class:"flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                        img {src:crate::CHECKMARK_URL, alt:"checkmark" }
                    }
                }
            }}


            span {class:"hidden md:flex px-0.5  font-[markoonefont] font-bold font-black
                text-sm lg:text-lg text-[var(--primary-color)]",
                {translations.translate(progress_data.heading.as_str()).unwrap_or(&progress_data.heading).as_str()}
            }
        }
    }
}

#[component]
fn ProgressInfoItemBar() -> Element {
    rsx! {
        div {class:"item-bar items-center justify-center"}
    }
}

#[component]
pub fn OrgInfo() -> Element {
    rsx! {"org_info"}
}

#[component]
pub fn SmtpSecure() -> Element {
    rsx! {"smtp_secure"}
}

#[component]
pub fn Passcode() -> Element {
    rsx! {"passcode"}
}

#[component]
pub fn InvalidState() -> Element {
    rsx! {"invalid"}
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct ProgressStateToUiRecord {
    progress_state: ConfigurationProgress,
    language_state: ProgressStateData,
    org_info_state: ProgressStateData,
    smtp_state: ProgressStateData,
    passcode_state: ProgressStateData,
}

impl ProgressStateToUiRecord {
    pub fn from_session_storage() -> WasmToolkitResult<Self> {
        let state = WINDOW
            .read()
            .get_session_storage_value("state")
            .map(|state_may_exist| {
                if let Some(state_found) = state_may_exist {
                    serde_json::from_str::<Self>(&state_found).unwrap_or_default()
                } else {
                    Self::default()
                }
            })?;

        // Write the state back in case the outcome of `state` is Self::default().
        // This is similar to just writing inside the map but just cleaner and in
        // case the state already existed this is very fast and done once on page load

        state.to_session_storage()?;

        Ok(state)
    }

    pub fn to_session_storage(&self) -> WasmToolkitResult<()> {
        let state = serde_json::to_string::<Self>(self).unwrap_or_default();
        WINDOW.read().set_session_storage_values("state", &state)?;

        Ok(())
    }
}

impl Default for ProgressStateToUiRecord {
    fn default() -> Self {
        Self {
            progress_state: ConfigurationProgress::Language,
            language_state: ProgressStateData::new_with_data("Language", "en-US"),
            org_info_state: ProgressStateData::new("Info"),
            smtp_state: ProgressStateData::new("Smtp(s)"),
            passcode_state: ProgressStateData::new("Passcode"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct ProgressStateData {
    heading: String,
    data: Option<String>,
    state: ProgressStateToUi,
}

impl ProgressStateData {
    fn new(heading: &str) -> Self {
        Self::new_with_data(heading, "")
    }

    fn new_with_data(heading: &str, data: &str) -> Self {
        Self {
            heading: heading.to_string(),
            data: if data.is_empty() {
                None
            } else {
                Some(data.to_string())
            },
            state: if heading == "Language" {
                ProgressStateToUi::Active
            } else {
                ProgressStateToUi::default()
            },
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
enum ProgressStateToUi {
    Active,
    #[default]
    Inactive,
    Processed,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
enum ConfigurationProgress {
    #[default]
    Language,
    OrgInfo,
    SmtpSecure,
    Passcode,
}

#[component]
fn FillSvg() -> Element {
    rsx! {
        svg {
            version: "1.1",
            view_box: "0 0 223.02 223.02",
            xmlns: "http://www.w3.org/2000/svg",
            circle {
                cx: "111.51",
                cy: "111.51",
                fill: "var(--primary-color)",
                r: "111.51",
                style: "paint-order:stroke fill markers",
            }
        }
    }
}

#[component]
fn StrokeFillSvg() -> Element {
    rsx! {
        svg {
            version: "1.1",
            view_box: "0 0 223.02 223.02",
            xmlns: "http://www.w3.org/2000/svg",
            circle {
                cx: "111.51",
                cy: "111.51",
                fill: "none",
                r: "98.694",
                stroke: "var(--primary-color)",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                stroke_width: "25.631",
                style: "paint-order:stroke fill markers",
            }
        }
    }
}
