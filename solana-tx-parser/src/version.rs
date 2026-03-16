use krill_common::{KrillError, KrillResult};
use serde::Deserialize;

use crate::{RpcResponse, SolanaTxParserUtils};

impl SolanaTxParserUtils {
    pub async fn get_version(&mut self) -> KrillResult<RpcResponse<RpcVersionInfo>> {
        let response_text = self.with_empty_params().send().await?;

        serde_json::from_str::<RpcResponse<RpcVersionInfo>>(&response_text).map_err(|error| {
            KrillError::HttpClient(format!(
                "Unable to parse the response for `{}`. Error: `{}`",
                self.method(),
                error
            ))
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RpcVersionInfo {
    pub solana_core: String,
    pub feature_set: Option<u32>,
}
