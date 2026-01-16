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
}
