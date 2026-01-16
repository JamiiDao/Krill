use krill_common::{ColorScheme, KrillError, KrillResult};

use crate::{KrillStorage, KrillStoreKeyspace};

impl KrillStorage {
    pub async fn set_branding_data(&self, color_scheme: ColorScheme) -> KrillResult<()> {
        let keyspace = self.branding_keyspace();

        self.set_op(
            keyspace,
            KrillStoreKeyspace::Branding.as_str(),
            color_scheme,
        )
        .await
    }

    pub async fn get_branding_data(&self) -> KrillResult<ColorScheme> {
        let keyspace = self.branding_keyspace();

        match self
            .get_op(keyspace, KrillStoreKeyspace::Branding.as_str())
            .await
        {
            Ok(bytes) => {
                bitcode::decode(&bytes).or(Err(KrillError::UnableToDeserializeBrandingData))
            }
            Err(error) => match error {
                KrillError::KeyNotFoundInStore(_) => {
                    let color_scheme = ColorScheme::new();

                    self.set_branding_data(color_scheme.clone()).await?;

                    Ok(color_scheme)
                }
                _ => Err(error),
            },
        }
    }
}
