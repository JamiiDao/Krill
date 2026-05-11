use web_sys::Document;

mod css_variables;
mod elements;
mod favicon;
mod focus;

mod dropzone;
pub use dropzone::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WasmDocument(Document);

impl WasmDocument {
    pub fn new(document: Document) -> Self {
        Self(document)
    }

    pub fn inner(&self) -> &Document {
        &self.0
    }

    pub fn set_page_title(&self, page_title: &str) {
        self.inner().set_title(page_title);
    }
}
