use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{WasmDocument, WasmToolkitError, WasmToolkitResult};

impl WasmDocument {
    pub fn set_focus_to_html_element(&self, element_id: &str) -> WasmToolkitResult<()> {
        self.get_html_element_by_id(element_id)?
            .ok_or(WasmToolkitError::UnableToFocusHtmlElement(
                element_id.to_string() + " id does not exist in the HTML document",
            ))?
            .focus()
            .or(Err(WasmToolkitError::UnableToFocusHtmlElement(
                element_id.to_string(),
            )))
    }

    pub fn get_focused_html_element(&self) -> WasmToolkitResult<Option<HtmlElement>> {
        self.inner()
            .active_element()
            .map(|value| {
                value
                    .dyn_into::<HtmlElement>()
                    .or(Err(WasmToolkitError::ElementIsNotHtmlElement))
            })
            .transpose()
    }

    pub fn remove_focus_entirely(&self) -> WasmToolkitResult<Option<()>> {
        self.get_focused_html_element()?
            .map(|focused| {
                focused
                    .blur()
                    .or(Err(WasmToolkitError::UnableToRemoveKeyboardFocus))
            })
            .transpose()
    }
}
