use wasm_bindgen::JsCast;
use web_sys::{CssStyleDeclaration, Element, HtmlElement};

use crate::{WasmDocument, WasmToolkitError, WasmToolkitResult};

impl WasmDocument {
    pub fn get_document_element(&self) -> WasmToolkitResult<Element> {
        self.inner()
            .document_element()
            .ok_or(WasmToolkitError::MissingDocumentElement)
    }

    pub fn get_document_html_element(&self) -> WasmToolkitResult<HtmlElement> {
        self.get_document_element()?
            .dyn_into::<HtmlElement>()
            .or(Err(WasmToolkitError::UnableToCastElementToHtmlElement))
    }

    pub fn root_css(&self) -> WasmToolkitResult<CssStyleDeclaration> {
        Ok(self.get_document_html_element()?.style())
    }

    pub fn set_css_variable(&self, key: &str, value: &str) -> WasmToolkitResult<()> {
        self.root_css()?
            .set_property(key, value)
            .or(Err(WasmToolkitError::UnableToSetCssProperty))
    }

    pub fn set_background_color(&self, value: &str) -> WasmToolkitResult<()> {
        self.set_css_variable("--background-color", value)
    }

    pub fn set_background_color_pitch_black(&self) -> WasmToolkitResult<()> {
        self.set_css_variable("--background-color", "#000000")
    }

    pub fn set_background_color_system_dark(&self) -> WasmToolkitResult<()> {
        self.set_background_color_pitch_black()
    }

    pub fn set_background_color_system_light(&self) -> WasmToolkitResult<()> {
        self.set_css_variable("--background-color", "#1a1a1a")
    }

    pub fn set_primary_color(&self, value: &str) -> WasmToolkitResult<()> {
        self.set_css_variable("--primary-color", value)
    }

    pub fn set_secondary_color(&self, value: &str) -> WasmToolkitResult<()> {
        self.set_css_variable("--secondary-color", value)
    }

    pub fn set_accent_color(&self, value: &str) -> WasmToolkitResult<()> {
        self.set_css_variable("--accent-color", value)
    }

    pub fn set_font_sans(&self, value: &str) -> WasmToolkitResult<()> {
        self.set_css_variable("--font-sans", value)
    }

    pub fn set_font_mono(&self, value: &str) -> WasmToolkitResult<()> {
        self.set_css_variable("--font-mono", value)
    }
}
