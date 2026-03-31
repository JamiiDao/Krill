use countries_iso3166::BC47LanguageInfo;
use fjall::Readable;
use krill_common::{KrillError, KrillResult};

use crate::KrillStorage;

const DEFAULT_LANGS: &[&str] = &[
    "en-US", "sw", "zh-Hans", "fr", "es", "pt-BR", "ar", "ru", "ja-JP", "de", "ko", "it", "vi-VN",
    "fa", "ur", "id", "tr", "uk", "hi",
];

impl KrillStorage {
    pub const KEYSPACE_SUPPORTED_LANGUAGES: &str = "SupportedLanguages";

    pub async fn set_supported_language(&self, bcp47_code: &str) -> KrillResult<()> {
        let code: BC47LanguageInfo = bcp47_code.into();

        if code == BC47LanguageInfo::UnsupportedLanguage {
            return Err(KrillError::LanguageNotValidBcp47Code(
                bcp47_code.to_string(),
            ));
        }

        let keyspace = self.languages_keyspace();

        self.set_op(keyspace, bcp47_code, code.english()).await
    }

    pub async fn get_supported_languages(&self) -> KrillResult<Vec<String>> {
        let read_tx = self.db().read_tx();
        let languages = read_tx
            .iter(self.languages_keyspace().as_ref())
            .map(|value| {
                let key = value
                    .key()
                    .or(Err(KrillError::Store("Read-Key".to_string())))?
                    .to_vec();

                String::from_utf8(key).or(Err(KrillError::UnableToDeserializeSupportedLanguages))
            })
            .collect::<KrillResult<Vec<String>>>()?;

        if languages.is_empty() {
            for lang in DEFAULT_LANGS {
                self.set_supported_language(lang).await?;
            }

            Ok(DEFAULT_LANGS
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<String>>())
        } else {
            Ok(languages)
        }
    }

    // Will ignore removing `en-US` as it is the default language for the server
    pub async fn remove_supported_languages(&self, bcp47_code: &str) -> KrillResult<()> {
        if bcp47_code == "en-US" {
            return Ok(());
        }

        self.remove_op(self.languages_keyspace(), bcp47_code).await
    }
}
