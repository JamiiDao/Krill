use countries_iso3166::{BC47LanguageInfo, SingleLanguageTranslationMap};
use dioxus::signals::ReadableExt;
use krill_common::{KrillError, KrillResult};
use serde::{Deserialize, Serialize};

use crate::WINDOW;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct SupportedLanguages(Vec<BC47LanguageInfo>);

impl SupportedLanguages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(bcp47_codes: Vec<String>) -> KrillResult<Self> {
        let mut langs = Vec::<BC47LanguageInfo>::new();

        for bcp47_code in bcp47_codes {
            let bcp47: BC47LanguageInfo = bcp47_code.as_str().into();

            if bcp47 == BC47LanguageInfo::UnsupportedLanguage {
                return Err(KrillError::LanguageNotValidBcp47Code(bcp47_code));
            }

            langs.push(bcp47);
        }

        langs.sort();

        Ok(Self(langs))
    }

    pub fn get_langs(&self) -> &[BC47LanguageInfo] {
        self.0.as_slice()
    }
}

impl Default for SupportedLanguages {
    fn default() -> Self {
        Self(vec![BC47LanguageInfo::EN_US])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Translations(SingleLanguageTranslationMap);

impl Translations {
    pub fn translate(&self, identifier: &str) -> String {
        if let Some(value) = self.0.get_translation(identifier) {
            value.to_string()
        } else {
            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            web_sys::console::error_1(
                &format!(
                    "Unable to find the translation for `{identifier}`. BCP-47 code: {}",
                    crate::SELECTED_LANGUAGE.read().code()
                )
                .into(),
            );

            identifier.to_string()
        }
    }

    pub async fn get_default_translation(page: &str) -> KrillResult<Self> {
        let lang = WINDOW
            .read()
            .language()
            .map_err(|error| KrillError::Transmit(error.to_string()))?;

        Self::get_translation(page, &lang).await
    }

    pub async fn get_translation(page: &str, target: &str) -> KrillResult<Self> {
        let mut language = target.to_string();

        let host_name = WINDOW
            .read()
            .origin()
            .map_err(|error| KrillError::Transmit(error.to_string()))?;

        let lang_path = Self::translation_path(page, &language);
        let mut url = host_name.clone() + lang_path.as_str();

        let mut response = Self::get_translation_text(&url).await?;

        if response.status() == 404 && language != "en-US" && language.contains('-') {
            language = language.split('-').next().unwrap_or("en-US").to_string();
            let lang_path_default = Self::translation_path(page, &language);
            url = host_name.clone() + lang_path_default.as_str();

            response = Self::get_translation_text(&url).await?;

            if response.status() == 404 && language != "en-US" && language != "en" {
                let lang_path_default = Self::translation_path_default(page);
                url = host_name + lang_path_default.as_str();

                response = Self::get_translation_text(&url).await?;
            }
        }

        if response.status() == 404 {
            Err(KrillError::FatalUi(
                "en-US translation not found yet it is the default".to_string(),
            ))
        } else {
            let body = response.text().await.map_err(|error| {
                KrillError::Transmit(
                    "Unable to parse the body of fetching a translation. Error: `{}`".to_string()
                        + error.to_string().as_str(),
                )
            })?;

            let translations =
                SingleLanguageTranslationMap::parse(&url, &body).map_err(|error| {
                    KrillError::Transmit(
                        "The file `".to_string()
                            + &url
                            + "` contains errors. Error message: `"
                            + error.to_string().as_str()
                            + "`.",
                    )
                })?;

            Ok(Self(translations))
        }
    }

    pub async fn get_translation_text(url: &str) -> KrillResult<reqwest::Response> {
        reqwest::Client::new()
            .get(url)
            .header("Content-Type", "text/plain")
            .send()
            .await
            .map_err(|error| KrillError::Transmit(error.to_string()))
    }

    pub fn translation_path(page_name: &str, language: &str) -> String {
        let language = if language == "en" { "en-US" } else { language };

        String::from("/assets/translations/") + page_name + "/" + language + ".bcp47"
    }

    pub fn translation_path_default(page_name: &str) -> String {
        Self::translation_path(page_name, "en-US")
    }
}
