use web_sys::Document;

mod css_variables;
mod elements;
mod focus;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WasmDocument(Document);

impl WasmDocument {
    pub fn new(document: Document) -> Self {
        Self(document)
    }

    pub fn inner(&self) -> &Document {
        &self.0
    }
}
