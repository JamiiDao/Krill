use wasm_bindgen::JsCast;
use web_sys::HtmlLinkElement;

use crate::{WasmDocument, WasmToolkitCommon, WasmToolkitError, WasmToolkitResult};

impl WasmDocument {
    pub fn set_favicon(&self, bytes: &[u8]) -> WasmToolkitResult<()> {
        let document = self.inner();

        // remove all existing favicons
        let icons = document
            .query_selector_all("link[rel*='icon']")
            .map_err(|error| {
                WasmToolkitError::parse_js_error(error, "Unable to query favicon links")
            })?;

        for index in 0..icons.length() {
            if let Some(node) = icons.item(index)
                && let Some(parent) = node.parent_node()
            {
                let _ = parent.remove_child(&node);
            }
        }

        let link = document
            .create_element("link")
            .map_err(|error| {
                WasmToolkitError::parse_js_error(error, "Unable to create favicon link")
            })?
            .dyn_into::<HtmlLinkElement>()
            .or(Err(WasmToolkitError::JsError {
                name: "Cast".to_string(),
                message: "Unable to cast HtmlLinkElement".to_string(),
            }))?;

        link.set_rel("icon");

        let media_type = WasmToolkitCommon::media_type(bytes);
        let base64_data = WasmToolkitCommon::bytes_to_css_base64(bytes);

        link.set_type(&media_type);

        // cache busting helps Firefox
        let href = format!("{}#{}", base64_data, js_sys::Date::now());

        link.set_href(&href);

        document
            .head()
            .ok_or(WasmToolkitError::JsError {
                name: "Head".to_string(),
                message: "Missing head".to_string(),
            })?
            .append_child(&link)
            .map_err(|error| WasmToolkitError::parse_js_error(error, "Unable to append favicon"))?;

        Ok(())
    }
}
