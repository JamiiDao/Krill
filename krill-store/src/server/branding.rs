use krill_common::{ColorScheme, KrillError, KrillResult};

use crate::KrillStorage;

impl KrillStorage {
    pub const KEYSPACE_BRANDING: &str = "Branding";

    pub async fn set_branding_data(&self, color_scheme: ColorScheme) -> KrillResult<()> {
        let keyspace = self.branding_keyspace();

        self.set_op(keyspace, Self::KEYSPACE_BRANDING, color_scheme)
            .await
    }

    pub async fn get_branding_data(&self) -> KrillResult<ColorScheme> {
        let bytes = self.get_branding_data_bytes().await?;

        bitcode::decode(&bytes).or(Err(KrillError::UnableToDeserializeBrandingData))
    }

    pub async fn get_branding_data_bytes(&self) -> KrillResult<Vec<u8>> {
        let keyspace = self.branding_keyspace();

        match self.get_op(keyspace, Self::KEYSPACE_BRANDING).await? {
            Some(bytes) => Ok(bytes),
            None => {
                let color_scheme = ColorScheme::new();

                self.set_branding_data(color_scheme.clone()).await?;

                Ok(bitcode::encode(&color_scheme))
            }
        }
    }
}
