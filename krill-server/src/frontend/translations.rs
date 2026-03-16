use countries_iso3166::SingleLanguageTranslationMap;
use dioxus::signals::ReadableExt;
use krill_common::{KrillError, KrillResult};

use crate::WINDOW;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Translations(SingleLanguageTranslationMap);

impl Translations {
    pub fn translate(&self, identifier: &str) -> Option<&String> {
        self.0.get_translation(identifier)
    }

    pub async fn get_translation(page: &str) -> KrillResult<Self> {
        let mut language = WINDOW
            .read()
            .language()
            .map_err(|error| KrillError::Transmit(error.to_string()))?;
        let host_name = WINDOW
            .read()
            .origin()
            .map_err(|error| KrillError::Transmit(error.to_string()))?;

        let lang_path = Self::translation_path(page, &language);
        let mut url = host_name.clone() + lang_path.as_str();

        #[cfg(debug_assertions)]
        tracing::info!("LANGUAGE PATH: {url}");

        let mut response = Self::get_translation_text(&url).await?;

        if response.status() == 404 && language != "en-US" && language.contains('-') {
            language = language.split('-').next().unwrap_or("en-US").to_string();
            let lang_path_default = Self::translation_path(page, &language);
            url = host_name.clone() + lang_path_default.as_str();
            #[cfg(debug_assertions)]
            tracing::info!("LANGUAGE NOT FOUND, TRYING... {url}");

            response = Self::get_translation_text(&url).await?;

            if response.status() == 404 && language != "en-US" && language != "en" {
                tracing::info!("LANGUAGE PATH NOT FOUND TRYING en-US default");
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

            #[cfg(debug_assertions)]
            tracing::info!("LANGUAGE Translations: {translations:?}");

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
