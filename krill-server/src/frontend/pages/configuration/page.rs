use core::fmt;

use countries_iso3166::BC47LanguageInfo;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_toolkit::WasmToolkitResult;

use crate::{
    ApiSecrets, LanguageView, Loader, OrgInfo, Passcode, Translations, Verification,
    NOTIFICATION_MANAGER, WINDOW,
};

#[component]
pub fn Configuration() -> Element {
    use_context_provider(|| Signal::new(ProgressStateToUiRecord::default()));
    use_context_provider(|| Signal::new(Translations::default()));

    let mut state_data = consume_context::<Signal<ProgressStateToUiRecord>>();
    let mut translations = consume_context::<Signal<Translations>>();
    let mut loading_langs = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            match ProgressStateToUiRecord::from_session_storage() {
                Ok(state_may_exist) => {
                    state_data.set(state_may_exist);
                }
                Err(error) => {
                    NOTIFICATION_MANAGER
                        .send_final(wasm_toolkit::NotificationType::Failure(error))
                        .await
                }
            }

            match Translations::get_translation(
                "configuration",
                state_data.read().language().code(),
            )
            .await
            {
                Ok(fetched_translations) => {
                    translations.set(fetched_translations);

                    loading_langs.set(false);
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
        if *loading_langs.read() {
            div { class: "flex w-full flex-col h-screen items-center justify-center",
                div { class: "flex dark:text-[var(--primary-color)] font-[headingfont] w-full
                items-center justify-center mb-10 text-2xl font-black",
                    "FETCHING DEFAULT LANGUAGE DATA"
                }
                Loader {}
            }
        } else {
            div { class: "flex flex-col min-h-screen w-[90dvw] self-center items-center justify-center text-center",
                div { class: "min-h-[10dvh] flex transition duration-1000 ease-in-out flex-col w-full justify-end items-center h-full p-1",
                    img {
                        class: "flex w-[35%] md:w-[20%] lg:w-[15%]",
                        src: state_data.read().progress_state.icon(),
                        alt: state_data.read().progress_state.icon_alt(),
                    }
                }

                div { class: "min-h-[5dvh] flex transition duration-1000 ease-in-out flex-col w-full justify-center items-center h-full p-2",
                    {progress_info(state_data, translations)}
                }

                div { class: "min-h-[50dvh] flex flex-col w-full justify-start items-center h-full",
                    match &state_data.read().progress_state {
                        ConfigurationProgress::Language => rsx! {
                            LanguageView {}
                        },
                        ConfigurationProgress::OrgInfo => rsx! {
                            OrgInfo {}
                        },
                        ConfigurationProgress::ApiSecrets => rsx! {
                            ApiSecrets {}
                        },
                        ConfigurationProgress::Passcode => rsx! {
                            Passcode {}
                        },
                        ConfigurationProgress::Verification => rsx! {
                            Verification {}
                        },
                    }
                }
            }
        }
    }
}

fn progress_info(
    progress_state_data: Signal<ProgressStateToUiRecord>,
    translations: Signal<Translations>,
) -> Element {
    rsx! {
        div { class: "flex w-full grid-column-7",
            {
                progress_info_item(
                    progress_state_data,
                    ConfigurationProgress::Language,
                    translations,
                )
            }
            ProgressInfoItemBar {}
            {
                progress_info_item(
                    progress_state_data,
                    ConfigurationProgress::OrgInfo,
                    translations,
                )
            }
            ProgressInfoItemBar {}
            {
                progress_info_item(
                    progress_state_data,
                    ConfigurationProgress::ApiSecrets,
                    translations,
                )
            }
            ProgressInfoItemBar {}
            {
                progress_info_item(
                    progress_state_data,
                    ConfigurationProgress::Passcode,
                    translations,
                )
            }
        }
    }
}

fn progress_info_item(
    mut selected_state: Signal<ProgressStateToUiRecord>,
    item_state: ConfigurationProgress,
    translations: Signal<Translations>,
) -> Element {
    rsx! {
        div {
            class: "item w-full transition duration-1000 ease-in-out hover:scale-110 flex rounded-full p-5 items-center justify-center cursor-pointer",
            onclick: move |_| {
                selected_state.write().progress_state = item_state;
            },

            if selected_state.read().progress_state == item_state {

                span { class: "flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                    FillSvg {}
                }
            }
            if selected_state.read().progress_state < item_state {

                span { class: "flex mr-1 max-w-[20px] min-w-[15px] w-[20%] rounded-full p-[1px]",
                    StrokeFillSvg {}
                }
            }
            if selected_state.read().progress_state > item_state {
                span { class: "flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                    img { src: crate::CHECKMARK_URL, alt: "checkmark" }
                }
            }

            span { class: "hidden md:flex px-0.5  font-[subheadingfont] font-bold font-black
                text-sm lg:text-lg dark:text-[var(--primary-color)]",
                {translations.read().translate(item_state.language_key())}
            }
        }
    }
}

#[component]
fn ProgressInfoItemBar() -> Element {
    rsx! {
        div { class: "item-bar items-center justify-center" }
    }
}

#[component]
pub fn InvalidState() -> Element {
    rsx! { "invalid" }
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct ProgressStateToUiRecord {
    pub progress_state: ConfigurationProgress,
    pub language: BC47LanguageInfo,
    pub org_info: CacheOrgInfo,
    pub domain_name: Option<String>,
    pub smtp_info: Option<String>,
    pub api_key: Option<String>,
    pub passcode: Option<String>,
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

    pub fn set_language(&mut self, code: BC47LanguageInfo) -> WasmToolkitResult<&mut Self> {
        self.language = code;

        self.to_session_storage()?;

        Ok(self)
    }

    pub fn set_org_name(&mut self, name: &str) -> &mut Self {
        self.org_info.name.replace(name.to_string());

        self
    }

    pub fn set_domain(&mut self, domain: &str) -> &mut Self {
        self.domain_name.replace(domain.to_string());

        self
    }

    pub fn set_support_mail(&mut self, support_mail: &str) -> &mut Self {
        self.org_info.support_mail.replace(support_mail.to_string());

        self
    }

    pub fn set_logo(&mut self, logo_bytes: Vec<u8>, mime: String) -> &mut Self {
        self.org_info.logo.replace((logo_bytes, mime));

        self
    }

    pub fn set_favicon(&mut self, favicon_bytes: Vec<u8>, mime: String) -> &mut Self {
        self.org_info.favicon.replace((favicon_bytes, mime));

        self
    }

    pub fn set_api_key(&mut self, api_key: &str) -> &mut Self {
        self.api_key.replace(api_key.to_string());

        self
    }

    pub fn set_smtps(&mut self, smtps: &str) -> &mut Self {
        self.smtp_info.replace(smtps.to_string());

        self
    }

    pub fn set_passcode(&mut self, passcode: String) -> &mut Self {
        self.passcode.replace(passcode);

        self
    }

    pub fn set_progress_state(&mut self, state: ConfigurationProgress) -> &mut Self {
        self.progress_state = state;

        self
    }

    pub(crate) fn transition(&mut self) -> WasmToolkitResult<ConfigurationProgress> {
        let state = match self.progress_state {
            ConfigurationProgress::Language => ConfigurationProgress::OrgInfo,
            ConfigurationProgress::OrgInfo => ConfigurationProgress::ApiSecrets,
            ConfigurationProgress::ApiSecrets => ConfigurationProgress::Passcode,
            ConfigurationProgress::Passcode => ConfigurationProgress::Verification,
            ConfigurationProgress::Verification => ConfigurationProgress::Verification,
        };

        self.progress_state = state;
        self.to_session_storage()?;

        Ok(state)
    }

    pub(crate) fn language(&self) -> BC47LanguageInfo {
        self.language
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct CacheOrgInfo {
    pub name: Option<String>,
    pub support_mail: Option<String>,
    pub logo: Option<(Vec<u8>, String)>,
    pub favicon: Option<(Vec<u8>, String)>,
    pub dominant_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
}

impl fmt::Debug for CacheOrgInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_hash = |byte_info: Option<&(Vec<u8>, String)>| -> Option<String> {
            byte_info
                .as_ref()
                .map(|(bytes, mime)| format!("Blake3({}).{}", blake3::hash(bytes), mime))
        };

        f.debug_struct("CacheOrgInfo")
            .field("name", &self.name)
            .field("support_mail", &self.support_mail)
            .field("logo", &as_hash(self.logo.as_ref()))
            .field("favicon", &as_hash(self.favicon.as_ref()))
            .field("dominant_color", &self.dominant_color)
            .field("secondary_color", &self.secondary_color)
            .field("accent_color", &self.accent_color)
            .finish()
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum ConfigurationProgress {
    #[default]
    Language,
    OrgInfo,
    ApiSecrets,
    Passcode,
    Verification,
}

impl ConfigurationProgress {
    fn language_key(&self) -> &str {
        match self {
            Self::Language => "language",
            Self::OrgInfo => "branding",
            Self::ApiSecrets => "apisecrets",
            Self::Passcode => "passcode",
            Self::Verification => "verification",
        }
    }

    fn icon_alt(&self) -> &str {
        match self {
            Self::Language => "Krill Shield Logo",
            Self::OrgInfo => "Branding",
            Self::ApiSecrets => "Api Secrets",
            Self::Passcode => "Passcode",
            Self::Verification => "Verification",
        }
    }

    fn icon(&self) -> Asset {
        match self {
            Self::Language => asset!("/assets/icons/language.svg"),
            Self::OrgInfo => asset!("/assets/icons/identity.svg"),
            Self::ApiSecrets => asset!("/assets/icons/network.svg"),
            Self::Passcode => asset!("/assets/icons/krill-shield-logo.svg"),
            Self::Verification => asset!("/assets/icons/krill-shield-logo.svg"),
        }
    }
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
