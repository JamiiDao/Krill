use core::fmt;
use std::collections::HashMap;

use crate::{KrillError, KrillResult};

pub type TranslationsMap = HashMap<SupportedLanguages, &'static str>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Translator(TranslationsMap);

impl Translator {
    pub fn new(translations: &'static str) -> KrillResult<Self> {
        let mut map = HashMap::default();

        translations
            .lines()
            .into_iter()
            .try_for_each(|current_line| {
                let current_line = current_line.trim();
                if !current_line.is_empty() {
                    current_line
                        .split_once("=")
                        .map(|(code, translation)| {
                            let language = SupportedLanguages::from_bcp47(code.trim())
                                .ok_or(KrillError::LanguageNotValidBcp47Code(code))?;

                            map.insert(language, translation.trim());

                            Ok::<(), KrillError>(())
                        })
                        .transpose()?
                        .ok_or(KrillError::InvalidLanguageEntry(current_line))?;
                }

                Ok::<(), KrillError>(())
            })?;

        // Check if all languages have been covered by the translation list
        // Self::check_all(&map)?;

        Ok(Self(map))
    }

    fn check_all(map: &TranslationsMap) -> KrillResult<()> {
        map.get(&SupportedLanguages::English)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::English,
            ))?;
        map.get(&SupportedLanguages::Kiswahili)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Kiswahili,
            ))?;
        map.get(&SupportedLanguages::SimplifiedChinese).ok_or(
            KrillError::LanguageTranslationNotFound(SupportedLanguages::SimplifiedChinese),
        )?;
        map.get(&SupportedLanguages::French)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::French,
            ))?;
        map.get(&SupportedLanguages::Spanish)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Spanish,
            ))?;
        map.get(&SupportedLanguages::BrazilianPortuguese).ok_or(
            KrillError::LanguageTranslationNotFound(SupportedLanguages::BrazilianPortuguese),
        )?;
        map.get(&SupportedLanguages::Arabic)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Arabic,
            ))?;
        map.get(&SupportedLanguages::Russian)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Russian,
            ))?;
        map.get(&SupportedLanguages::Japanese)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Japanese,
            ))?;
        map.get(&SupportedLanguages::German)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::German,
            ))?;
        map.get(&SupportedLanguages::Korean)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Korean,
            ))?;
        map.get(&SupportedLanguages::Italian)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Italian,
            ))?;
        map.get(&SupportedLanguages::Vietnamese)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Vietnamese,
            ))?;
        map.get(&SupportedLanguages::PersianFarsi).ok_or(
            KrillError::LanguageTranslationNotFound(SupportedLanguages::PersianFarsi),
        )?;
        map.get(&SupportedLanguages::Urdu)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Urdu,
            ))?;
        map.get(&SupportedLanguages::Indonesian)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Indonesian,
            ))?;
        map.get(&SupportedLanguages::Turkish)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Turkish,
            ))?;
        map.get(&SupportedLanguages::Ukrainian)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Ukrainian,
            ))?;
        map.get(&SupportedLanguages::Hindi)
            .ok_or(KrillError::LanguageTranslationNotFound(
                SupportedLanguages::Hindi,
            ))?;

        Ok(())
    }

    pub fn translate_to(&self, language: SupportedLanguages) -> &'static str {
        self.0.get(&language).unwrap() // The language should already exist in the in-memory DB
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum SupportedLanguages {
    Kiswahili,
    English,
    SimplifiedChinese,
    French,
    Spanish,
    BrazilianPortuguese,
    Arabic,
    Russian,
    Japanese,
    German,
    Korean,
    Italian,
    Vietnamese,
    PersianFarsi,
    Urdu,
    Indonesian,
    Turkish,
    Ukrainian,
    Hindi,
}

impl SupportedLanguages {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedLanguages::Kiswahili => "Kiswahili (Swahili)",
            SupportedLanguages::English => "English",
            SupportedLanguages::SimplifiedChinese => "汉语 (Chinese)",
            SupportedLanguages::French => "Français (French)",
            SupportedLanguages::Spanish => "Español (Spanish)",
            SupportedLanguages::BrazilianPortuguese => "Português (Portuguese - Brazil)",
            SupportedLanguages::Arabic => "العربية (Arabic)",
            SupportedLanguages::Russian => "Русский (Russian)",
            SupportedLanguages::Japanese => "日本語 (Japanese)",
            SupportedLanguages::German => "Deutsch (German)",
            SupportedLanguages::Korean => "한국어 (Korean)",
            SupportedLanguages::Italian => "Italiano (Italian)",
            SupportedLanguages::Vietnamese => "Tiếng Việt (Vietnamese)",
            SupportedLanguages::PersianFarsi => "فارسی (Persian/Farsi)",
            SupportedLanguages::Urdu => "اردو (Urdu)",
            SupportedLanguages::Indonesian => "Bahasa Indonesia (Indonesian)",
            SupportedLanguages::Turkish => "Türkçe (Turkish)",
            SupportedLanguages::Ukrainian => "Українська (Ukrainian)",
            SupportedLanguages::Hindi => "हिन्दी (Hindi)",
        }
    }

    /// Returns the standard BCP-47 language code for this variant
    pub fn bcp47_code(&self) -> &'static str {
        match self {
            SupportedLanguages::Kiswahili => "sw",
            SupportedLanguages::English => "en",
            SupportedLanguages::SimplifiedChinese => "zh-Hans",
            SupportedLanguages::French => "fr",
            SupportedLanguages::Spanish => "es",
            SupportedLanguages::BrazilianPortuguese => "pt-BR",
            SupportedLanguages::Arabic => "ar",
            SupportedLanguages::Russian => "ru",
            SupportedLanguages::Japanese => "ja",
            SupportedLanguages::German => "de",
            SupportedLanguages::Korean => "ko",
            SupportedLanguages::Italian => "it",
            SupportedLanguages::Vietnamese => "vi",
            SupportedLanguages::PersianFarsi => "fa",
            SupportedLanguages::Urdu => "ur",
            SupportedLanguages::Indonesian => "id",
            SupportedLanguages::Turkish => "tr",
            SupportedLanguages::Ukrainian => "uk",
            SupportedLanguages::Hindi => "hi",
        }
    }

    /// closest SupportedLanguages variant from a BCP-47 code
    pub fn from_bcp47(code: &str) -> Option<Self> {
        let code = code.to_lowercase();

        let language = match code.as_str() {
            code if code.starts_with("en") => SupportedLanguages::English,
            code if code.starts_with("zh") => SupportedLanguages::SimplifiedChinese,
            code if code.starts_with("fr") => SupportedLanguages::French,
            code if code.starts_with("es") => SupportedLanguages::Spanish,
            code if code.starts_with("pt") => SupportedLanguages::BrazilianPortuguese,
            code if code.as_bytes() == SupportedLanguages::Kiswahili.bcp47_code().as_bytes() => {
                SupportedLanguages::Kiswahili
            }
            code if code.as_bytes() == SupportedLanguages::Arabic.bcp47_code().as_bytes() => {
                SupportedLanguages::Arabic
            }
            code if code.as_bytes() == SupportedLanguages::Russian.bcp47_code().as_bytes() => {
                SupportedLanguages::Russian
            }
            code if code.as_bytes() == SupportedLanguages::Japanese.bcp47_code().as_bytes() => {
                SupportedLanguages::Japanese
            }
            code if code.as_bytes() == SupportedLanguages::German.bcp47_code().as_bytes() => {
                SupportedLanguages::German
            }
            code if code.as_bytes() == SupportedLanguages::Korean.bcp47_code().as_bytes() => {
                SupportedLanguages::Korean
            }
            code if code.as_bytes() == SupportedLanguages::Italian.bcp47_code().as_bytes() => {
                SupportedLanguages::Italian
            }
            code if code.as_bytes() == SupportedLanguages::Vietnamese.bcp47_code().as_bytes() => {
                SupportedLanguages::Vietnamese
            }
            code if code.as_bytes() == SupportedLanguages::PersianFarsi.bcp47_code().as_bytes() => {
                SupportedLanguages::PersianFarsi
            }
            code if code.as_bytes() == SupportedLanguages::Urdu.bcp47_code().as_bytes() => {
                SupportedLanguages::Urdu
            }
            code if code.as_bytes() == SupportedLanguages::Indonesian.bcp47_code().as_bytes() => {
                SupportedLanguages::Indonesian
            }
            code if code.as_bytes() == SupportedLanguages::Turkish.bcp47_code().as_bytes() => {
                SupportedLanguages::Turkish
            }
            code if code.as_bytes() == SupportedLanguages::Ukrainian.bcp47_code().as_bytes() => {
                SupportedLanguages::Ukrainian
            }
            code if code.as_bytes() == SupportedLanguages::Hindi.bcp47_code().as_bytes() => {
                SupportedLanguages::Hindi
            }
            _ => return Option::None,
        };

        Some(language)
    }
}

impl fmt::Display for SupportedLanguages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
