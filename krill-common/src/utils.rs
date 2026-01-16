use camino::Utf8PathBuf;

pub struct KrillUtils;

impl KrillUtils {
    pub fn array_of_bytes_to_hex(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|byte| format!("{:0x?} ", byte))
            .collect::<String>()
            .trim()
            .to_string()
    }

    #[cfg(feature = "home-dir")]
    pub async fn create_recursive_dir(path: &Utf8PathBuf) -> crate::KrillResult<()> {
        use async_fs::DirBuilder;

        if let Some(error) = DirBuilder::new().recursive(true).create(path).await.err() {
            use std::io::ErrorKind;

            if error.kind() != ErrorKind::AlreadyExists {
                return Err(crate::KrillError::Io(error.kind()));
            }
        }

        Ok(())
    }

    #[cfg(feature = "home-dir")]
    pub async fn krill_dir() -> crate::KrillResult<Utf8PathBuf> {
        let mut db_dir = blocking::unblock(move || {
            dirs::home_dir()
                .map(|value| {
                    Utf8PathBuf::from_path_buf(value).or(Err(crate::KrillError::HomeDirPathNotUtf8))
                })
                .transpose()?
                .ok_or(crate::KrillError::UnableToFindHomeDirectory)
        })
        .await?;

        db_dir.push(".Krill");

        Ok(db_dir)
    }
}
