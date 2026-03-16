use dioxus::prelude::*;

#[cfg(feature = "server")]
use {
    async_dup::Arc,
    async_fs::File,
    async_lock::RwLock,
    countries_iso3166::BC47LanguageInfo,
    futures_lite::{io::BufReader, AsyncBufReadExt},
    krill_common::{KrillError, KrillResult},
    std::{
        collections::HashMap,
        path::{Path, PathBuf},
        sync::OnceLock,
    },
};

#[cfg(feature = "server")]
const TRANSLATIONS_PATH: &str = "./assets/translations";

#[cfg(feature = "server")]
const SUPPORTED_LANGUAGES_FILE_NAME: &str = "languages";

#[cfg(feature = "server")]
pub(crate) static SUPPORTED_TRANSLATIONS_LIST: OnceLock<SupportedTranslationsDir> = OnceLock::new();

#[server]
pub async fn langs_with_translations() -> ServerFnResult<Vec<String>> {
    // let outcome = SUPPORTED_TRANSLATIONS_LIST
    //     .get()
    //     .as_ref()
    //     .ok_or(ServerFnError::ServerError {
    //         message: "`SUPPORTED_TRANSLATIONS_LIST` not initialized yet!".to_string(),
    //         code: 500,
    //         details: None,
    //     })?
    //     .langs()
    //     .await;

    // tracing::info!("POST LANGS: {outcome:?}");

    Ok(vec!["zh-Hans".to_string()])
}

#[cfg(feature = "server")]
pub fn init_translations() -> KrillResult<()> {
    futures_lite::future::block_on(async move {
        let init = SupportedTranslationsDir::new()?;

        init.load_supported_languages().await?;

        if SUPPORTED_TRANSLATIONS_LIST.set(init).is_err() {
            tracing::error!("`SUPPORTED_TRANSLATIONS_LIST` is already initialized!");
        }

        Ok(())
    })
}

#[cfg(feature = "server")]
#[derive(Debug, Default)]
pub struct SupportedTranslationsDir {
    path: PathBuf,
    langs: Arc<RwLock<Vec<String>>>,
    paths: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[cfg(feature = "server")]
impl SupportedTranslationsDir {
    pub fn new() -> KrillResult<Self> {
        let path = Path::new(TRANSLATIONS_PATH)
            .canonicalize()
            .map_err(|error| KrillError::Io(error.kind()))?;

        Ok(Self {
            path,
            ..Default::default()
        })
    }

    pub async fn load_supported_languages(&self) -> KrillResult<()> {
        let mut path = self.path.clone();
        path.push(SUPPORTED_LANGUAGES_FILE_NAME);

        let file = File::open(path).await?;
        let mut reader = BufReader::new(file);

        let mut buf = String::new();
        let mut langs = Vec::new();

        loop {
            buf.clear();

            if reader.read_line(&mut buf).await? == 0 {
                break; // EOF
            }

            let line = buf.trim();

            let bcp47: BC47LanguageInfo = line.into();

            if bcp47 == BC47LanguageInfo::UnsupportedLanguage {
                return Err(KrillError::LanguageNotValidBcp47Code(line.to_string()));
            }

            langs.push(line.to_string());
        }

        self.langs.write().await.extend(langs);

        Ok(())
    }

    pub async fn load_translation_paths(&self) -> KrillResult<()> {
        let mut storage = self.paths.write().await;

        let outcome = dir_meta::DirMetadata::new(self.path.to_str().unwrap_or(TRANSLATIONS_PATH))
            .async_dir_metadata()
            .await?;

        // Iterate over the files

        for file in outcome.files() {
            let canonicalize = file.path().canonicalize()?;
            let stripped = canonicalize
                .strip_prefix(&self.path)
                .map_err(|error| KrillError::Transmit(error.to_string()))?;

            let mut components = stripped
                .components()
                .map(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .map(|value| value.to_string())
                        .ok_or("Path is not valid UTF-8.".to_string())
                })
                .collect::<Result<Vec<String>, String>>()
                .map_err(|error| KrillError::InvalidLanguageTranslationPath(error))?;

            if components.len() != 2 {
                return Err(KrillError::InvalidLanguageTranslationPath(format!("The path for a translation file `{stripped:?}` is invalid. There can only be one subfolder in the `translations` directory")));
            }

            let (key, value) = (components.remove(0), components.remove(0));
            storage.entry(key).or_default().push(value);
        }

        Ok(())
    }

    pub async fn langs(&self) -> Vec<String> {
        self.langs.read().await.clone()
    }

    pub async fn paths(&self) -> async_lock::RwLockReadGuard<'_, HashMap<String, Vec<String>>> {
        self.paths.read().await
    }
}
