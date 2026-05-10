use wasm_bindgen::JsCast;
use web_sys::HtmlLinkElement;

use crate::{WasmDocument, WasmToolkitCommon, WasmToolkitError, WasmToolkitResult};

impl WasmDocument {
    pub fn set_favicon(&self, bytes: &[u8]) -> WasmToolkitResult<()> {
        let selected = if let Some(value) = self
            .inner()
            .query_selector("link[rel*='icon']")
            .map_err(|error| {
                WasmToolkitError::parse_js_error(
                    error,
                    "Unable to find `link[rel*='icon']` in order to set the favicon",
                )
            })? {
            value
        } else {
            let element = self
                .inner()
                .create_element("link")
                .map_err(|error| {
                    WasmToolkitError::parse_js_error(
                        error,
                        "Unable to create a `link[rel*='icon']` element in the document",
                    )
                })?
                .dyn_into::<HtmlLinkElement>()
                .or(Err(WasmToolkitError::JsError {
                    name: "Cast".to_string(),
                    message: "Unable to cast to `HtmlLinkElement`".to_string(),
                }))?;

            element.set_rel("icon");

            self.inner()
                .head()
                .ok_or(WasmToolkitError::JsError {
                    name: "Head tag".to_string(),
                    message: "Unable to find the `head` tag in the document".to_string(),
                })?
                .append_child(&element)
                .map_err(|error| {
                    WasmToolkitError::parse_js_error(
                        error,
                        "Unable to create a `link[rel*='icon']` element in the document",
                    )
                })?;

            element.into()
        };

        let selected =
            selected
                .dyn_into::<HtmlLinkElement>()
                .or(Err(WasmToolkitError::JsError {
                    name: "Cast".to_string(),
                    message: "Unable to cast to `HtmlLinkElement`".to_string(),
                }))?;

        let media_type = WasmToolkitCommon::media_type(bytes);
        let base64_data = WasmToolkitCommon::bytes_to_css_base64(bytes);

        selected.set_rel("icon");
        selected.set_type(&media_type);
        selected.set_href(&base64_data);

        Ok(())
    }
}
