mod location;
mod match_media;

use crate::{WasmToolkitError, WasmToolkitResult};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WasmWindow(web_sys::Window);
impl WasmWindow {
    pub fn new() -> WasmToolkitResult<Self> {
        web_sys::window()
            .map(Self)
            .ok_or(WasmToolkitError::WindowNotFound)
    }

    pub fn inner(&self) -> &web_sys::Window {
        &self.0
    }

    pub fn clone_window(&self) -> web_sys::Window {
        self.0.clone()
    }
}
