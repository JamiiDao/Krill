use std::borrow::Cow;

use file_format::{FileFormat, Kind as FileFormatKind};
use js_sys::JSON;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::DomException;

use crate::{WasmToolkitError, WasmToolkitResult};

pub struct WasmToolkitCommon;

impl WasmToolkitCommon {
    pub fn stringify(obj: &JsValue) -> WasmToolkitResult<String> {
        JSON::stringify(obj)
            .or(Err(WasmToolkitError::UnableToStringifyJsValue))
            .map(|value| value.as_string().ok_or(WasmToolkitError::JsStringNotValid))?
    }

    /// Converts the JsValue error into a string.
    /// First checks if the error is a [DomException::message] first
    /// if not try to convert the error using [JSON::stringify] and if that fails
    /// just return an [WasmToolkitError::UnableToStringifyJsValue] error
    pub fn exception_or_stringify(error: &JsValue) -> String {
        if let Some(exception) = Self::as_dom_exception(error) {
            exception.message()
        } else {
            Self::ok_err_as_string(Self::stringify(error))
        }
    }

    pub fn as_dom_exception(error: &JsValue) -> Option<DomException> {
        error.dyn_ref::<DomException>().cloned()
    }

    pub fn ok_err_as_string(result: WasmToolkitResult<String>) -> String {
        match result {
            Ok(value) => value,
            Err(error) => error.to_string(),
        }
    }

    /*
        Must not be empty
    Can contain:
    letters, numbers
    ., _, +, -
    Must not start or end with .
    Must not have consecutive dots */
    pub fn is_email_valid(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();

        if parts.len() != 2 {
            return false;
        }

        let (local, domain) = (parts[0], parts[1]);

        if local.is_empty() {
            return false;
        }

        let mut prev_dot = false;

        for (i, c) in local.chars().enumerate() {
            if !(c.is_ascii_alphanumeric() || c == '.') {
                return false;
            }

            // no leading/trailing dot
            if (i == 0 || i == local.len() - 1) && c == '.' {
                return false;
            }

            // no consecutive dots
            if c == '.' {
                if prev_dot {
                    return false;
                }
                prev_dot = true;
            } else {
                prev_dot = false;
            }
        }

        if !domain.contains('.') {
            return false;
        }

        let domain_parts: Vec<&str> = domain.split('.').collect();

        if domain_parts.iter().any(|p| p.is_empty()) {
            return false;
        }

        if domain_parts.last().unwrap().len() < 2 {
            return false;
        }

        if !domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.')
        {
            return false;
        }

        true
    }

    pub fn bytes_to_css_base64(bytes: &[u8]) -> String {
        use base64ct::{Base64, Encoding};

        let encoded = Base64::encode_string(bytes);

        let media_type = Self::media_type(bytes);

        format!("data:{};base64,{}", media_type, encoded)
    }

    pub fn media_type(bytes: &[u8]) -> Cow<'_, str> {
        Self::to_file_format(bytes).media_type().to_string().into()
    }

    pub fn media_type_extension(bytes: &[u8]) -> Cow<'_, str> {
        Self::to_file_format(bytes).extension().to_string().into()
    }

    pub fn to_file_format_kind(bytes: &[u8]) -> FileFormatKind {
        Self::to_file_format(bytes).kind()
    }

    pub fn to_file_format(bytes: &[u8]) -> FileFormat {
        FileFormat::from_bytes(bytes)
    }
}
