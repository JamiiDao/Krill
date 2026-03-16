use std::sync::LazyLock;

use krill_common::{KrillError, KrillResult};
use reqwest::RequestBuilder;
use serde::Deserialize;

#[allow(clippy::redundant_closure)]
static SOLANA_REQWEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

#[derive(Debug, Clone)]
pub struct SolanaTxParserUtils {
    uri: JsonRpcCluster,
    method: String,
    body: jzon::JsonValue,
}

impl SolanaTxParserUtils {
    pub fn new(uri: JsonRpcCluster) -> Self {
        Self {
            uri,
            method: "getVersion".to_string(),
            body: jzon::array![],
        }
    }

    pub fn set_method(&mut self, method: &str) -> &mut Self {
        self.method = method.to_string();

        self
    }

    pub fn set_params(&mut self, params: jzon::JsonValue) -> &mut Self {
        let body = jzon::object! {
            "jsonrpc": "2.0",
            "id": 1,
            "method": self.method.as_str(),
            params: params
        };

        self.body = body;

        self
    }

    pub fn with_empty_params(&mut self) -> &mut Self {
        let body = jzon::object! {
            "jsonrpc": "2.0",
            "id": 1,
            "method": self.method.as_str(),
        };

        self.body = body;

        self
    }

    pub fn uri(&self) -> &JsonRpcCluster {
        &self.uri
    }

    pub fn body(&self) -> String {
        self.body_raw().to_string()
    }

    pub fn body_raw(&self) -> &jzon::JsonValue {
        &self.body
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn get_client(&self) -> RequestBuilder {
        SOLANA_REQWEST_CLIENT
            .post(self.uri().as_str())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
    }

    pub async fn send(&self) -> KrillResult<String> {
        self.get_client().body(
               self.body()
            ).send().await.map_err(|error| {
                KrillError::HttpClient(error.to_string())
            })?.text().await.map_err(|error| {
                KrillError::HttpClient(format!("HTTP ERROR: The result of `{}` Solana JSONRPC method is not a JSON string. Error: {}", self.method(),error))
            })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub id: u8,
    pub error: Option<RpcError>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RpcVersionInfo {
    pub solana_core: String,
    pub feature_set: Option<u32>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum JsonRpcCluster {
    Devnet,
    Testnet,
    Custom(String),
}

impl JsonRpcCluster {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Devnet => "https://api.devnet.solana.com",
            Self::Testnet => "https://api.testent.solana.com",
            Self::Custom(uri) => uri.as_str(),
        }
    }
}
