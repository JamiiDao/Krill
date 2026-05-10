use base64ct::{Base64,  Encoding};
use dioxus::prelude::*;
use krill_common::OrganizationInfo;
use wasm_toolkit::{WasmToolkitError, WasmToolkitResult};

use crate::WINDOW;

pub struct OrgCacheOps;

impl OrgCacheOps {
    pub const LOCAL_STORAGE_KEY: &str = "org_info";

    pub fn set_org_info(org_info: &OrganizationInfo) -> WasmToolkitResult<()> {
        let encoded_value = bitcode::encode(org_info);
        let encoded_value_base64 = Base64::encode_string(&encoded_value);

        WINDOW
            .read()
            .set_local_storage_values(Self::LOCAL_STORAGE_KEY, &encoded_value_base64)
    }

    pub fn get_org_info() -> WasmToolkitResult<OrganizationInfo> {
        Ok(WINDOW
            .read().get_local_storage_value(Self::LOCAL_STORAGE_KEY)?
            .map(|value| {
                let from_base64 = Base64::decode_vec(&value).or(Err(WasmToolkitError::JsError { name: "Local storage data to Bse64 error".to_string(), message: "Unable to decode the OrganizationInfo from the Base64 string stored in local storage. The Base64 string might be corrupted".to_string() }))?;
                
                bitcode::decode::<OrganizationInfo>(&from_base64).or(Err(WasmToolkitError::JsError { name: "Local storage decode error".to_string(), message: "Unable to decode the OrganizationInfo from the bytes stored in local storage. Those bytes might be corrupted".to_string() }))

            }).transpose()?.unwrap_or_default())
    }
}
