use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{WasmDocument, WasmToolkitError, WasmToolkitResult};

impl WasmDocument {
    pub fn get_html_element_by_id(
        &self,
        element_id: &str,
    ) -> WasmToolkitResult<Option<HtmlElement>> {
        self.inner()
            .get_element_by_id(element_id)
            .map(|value| {
                value
                    .dyn_into::<HtmlElement>()
                    .or(Err(WasmToolkitError::ElementIsNotHtmlElement))
            })
            .transpose()
    }

    pub fn get_element_by_id(&self, element_id: &str) -> Option<web_sys::Element> {
        self.inner().get_element_by_id(element_id)
    }
}
