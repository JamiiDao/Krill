use crate::{WasmToolkitError, WasmToolkitResult, WasmWindow};

impl WasmWindow {
    /// On iOS 12 and below the second part of the BCP-47 code is corrected
    /// to uppercase
    pub fn language(&self) -> WasmToolkitResult<String> {
        self.navigator()
            .language()
            .map(|language| {
                if let Some((first, second)) = language.split_once("-") {
                    first.to_string() + "-" + second.to_uppercase().as_str()
                } else {
                    language
                }
            })
            .ok_or(WasmToolkitError::BrowserLanguageNotFound)
    }
}
