use web_sys::Storage;

use crate::{WasmToolkitError, WasmToolkitResult, WasmWindow};

impl WasmWindow {
    pub fn session_storage(&self) -> WasmToolkitResult<Storage> {
        self.inner()
            .session_storage()
            .map_err(|error| {
                WasmToolkitError::parse_js_error(error, "Get sessionStorage securityError")
            })?
            .ok_or(WasmToolkitError::SessionStorageNotFound)
    }

    pub fn set_session_storage_values(&self, key: &str, value: &str) -> WasmToolkitResult<()> {
        Self::set_storage_values(
            self.session_storage()?,
            key,
            value,
            "Set sessionStorage item",
        )
    }

    pub fn get_session_storage_value(&self, key: &str) -> WasmToolkitResult<Option<String>> {
        Self::get_storage_value(self.session_storage()?, key, "Get sessionStorage item")
    }

    pub fn remove_session_storage_item(&self, key: &str) -> WasmToolkitResult<()> {
        Self::remove_storage_item(self.session_storage()?, key, "Remove sessionStorage item")
    }

    pub fn clear_session_storage(&self) -> WasmToolkitResult<()> {
        Self::clear_storage(self.session_storage()?, "Get sessionStorage clear method")
    }

    pub async fn all_session_storage_values(&self) -> WasmToolkitResult<Vec<(String, String)>> {
        let storage = self.session_storage()?;
        let length = storage.length().map_err(|error| {
            WasmToolkitError::parse_js_error(error, "Get sessionStorage length")
        })?;

        (0..length)
            .map(|index| {
                let item_name = storage
                    .key(index)
                    .map_err(|error| {
                        WasmToolkitError::parse_js_error(error, "Get sessionStorage key")
                    })?
                    .ok_or(WasmToolkitError::JsError {
                        name: "Indexed position not found".to_string(),
                        message: index.to_string() + " index out of bounds in the session storage",
                    })?;

                let value = self.get_session_storage_value(&item_name)?.ok_or(
                    WasmToolkitError::JsError {
                        name: "Indexed item not found".to_string(),
                        message: item_name.clone()
                            + " not found in the session storage yet it is indexed",
                    },
                )?;

                Ok((item_name, value))
            })
            .collect::<WasmToolkitResult<Vec<(String, String)>>>()
    }
}

impl WasmWindow {
    pub fn local_storage(&self) -> WasmToolkitResult<Storage> {
        self.inner()
            .local_storage()
            .map_err(|error| {
                WasmToolkitError::parse_js_error(error, "Get localStorage securityError")
            })?
            .ok_or(WasmToolkitError::LocalStorageNotFound)
    }

    pub fn set_local_storage_values(&self, key: &str, value: &str) -> WasmToolkitResult<()> {
        Self::set_storage_values(self.local_storage()?, key, value, "Set localStorage item")
    }

    pub fn get_local_storage_value(&self, key: &str) -> WasmToolkitResult<Option<String>> {
        Self::get_storage_value(self.local_storage()?, key, "Get localStorage item")
    }

    pub fn remove_local_storage_item(&self, key: &str) -> WasmToolkitResult<()> {
        Self::remove_storage_item(self.local_storage()?, key, "Remove localStorage item")
    }

    pub async fn clear_local_storage(&self) -> WasmToolkitResult<()> {
        Self::clear_storage(self.local_storage()?, "Clear localStorage clear method")
    }

    pub async fn all_local_storage_values(&self) -> WasmToolkitResult<Vec<(String, String)>> {
        let storage = self.local_storage()?;
        let length = storage
            .length()
            .map_err(|error| WasmToolkitError::parse_js_error(error, "Get localStorage length"))?;

        (0..length)
            .map(|index| {
                let item_name = storage
                    .key(index)
                    .map_err(|error| {
                        WasmToolkitError::parse_js_error(error, "Get localStorage key")
                    })?
                    .ok_or(WasmToolkitError::JsError {
                        name: "Indexed position not found".to_string(),
                        message: index.to_string() + " index out of bounds in the local storage",
                    })?;

                let value =
                    self.get_local_storage_value(&item_name)?
                        .ok_or(WasmToolkitError::JsError {
                            name: "Indexed item not found".to_string(),
                            message: item_name.clone()
                                + " not found in the local storage yet it is indexed",
                        })?;

                Ok((item_name, value))
            })
            .collect::<WasmToolkitResult<Vec<(String, String)>>>()
    }
}

impl WasmWindow {
    pub fn set_storage_values(
        storage: Storage,
        key: &str,
        value: &str,
        fallback_error: &str,
    ) -> WasmToolkitResult<()> {
        storage
            .set_item(key, value)
            .map_err(|error| WasmToolkitError::parse_js_error(error, fallback_error))
    }

    pub fn get_storage_value(
        storage: Storage,
        key: &str,
        fallback_error: &str,
    ) -> WasmToolkitResult<Option<String>> {
        storage
            .get_item(key)
            .map_err(|error| WasmToolkitError::parse_js_error(error, fallback_error))
    }

    pub fn remove_storage_item(
        storage: Storage,
        key: &str,
        fallback_error: &str,
    ) -> WasmToolkitResult<()> {
        storage
            .remove_item(key)
            .map_err(|error| WasmToolkitError::parse_js_error(error, fallback_error))
    }

    pub fn clear_storage(storage: Storage, fallback_error: &str) -> WasmToolkitResult<()> {
        storage
            .clear()
            .map_err(|error| WasmToolkitError::parse_js_error(error, fallback_error))
    }
}
