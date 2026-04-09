use countries_iso3166::BC47LanguageInfo;
use krill_common::{KrillError, KrillResult};

use crate::KrillStorage;

impl KrillStorage {
    pub const KEY_SERVER_LANGUAGE: &str = "ServerLanguage";

    pub async fn set_server_language(&self, bcp47_code: &str) -> KrillResult<()> {
        let parsed: BC47LanguageInfo = bcp47_code.into();

        if parsed == BC47LanguageInfo::UnsupportedLanguage {
            return Err(KrillError::LanguageNotValidBcp47Code(
                bcp47_code.to_string(),
            ));
        }

        let keyspace = self.org_info_keyspace();

        // Use enum to u16 auto-conversion
        self.set(keyspace, Self::KEY_SERVER_LANGUAGE, parsed.code())
            .await
    }

    pub async fn get_server_language(&self) -> KrillResult<BC47LanguageInfo> {
        let keyspace = self.org_info_keyspace();

        let lang_bytes = self.get(keyspace, Self::KEY_SERVER_LANGUAGE).await?;

        if let Some(bytes) = lang_bytes.as_ref() {
            let parsed = bitcode::decode::<String>(bytes).or(Err(KrillError::Store(
                "Unable to deserialize server language".to_string(),
            )))?;

            Ok(parsed.as_str().into())
        } else {
            Ok(BC47LanguageInfo::EN_US)
        }
    }
}
