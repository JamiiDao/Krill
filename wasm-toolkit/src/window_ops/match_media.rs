use std::{cell::RefCell, rc::Rc};

use async_channel::{Receiver, bounded};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, MediaQueryList};

use crate::{WasmToolkitCommon, WasmToolkitError, WasmToolkitResult, WasmWindow};

impl WasmWindow {
    pub fn match_media(&self, query: &str) -> WasmToolkitResult<MediaQueryList> {
        self.inner()
            .match_media(query)
            .map_err(|error| {
                let outcome = WasmToolkitCommon::exception_or_stringify(&error);

                WasmToolkitError::Op(outcome)
            })?
            .ok_or(WasmToolkitError::MatchMediaQueryUnsupported)
    }

    pub fn query_dark_mode(&self) -> WasmToolkitResult<MediaQueryList> {
        self.match_media("(prefers-color-scheme: dark)")
    }

    pub fn is_dark_mode(&self) -> WasmToolkitResult<bool> {
        Ok(self.query_dark_mode()?.matches())
    }

    pub async fn watch_dark_mode(&self) -> WasmToolkitResult<Receiver<bool>> {
        let (sender, receiver) = bounded::<bool>(1);

        let query = Rc::new(RefCell::new(self.query_dark_mode()?));
        let query_spawned = query.clone();

        let callback = Closure::wrap(Box::new(move |_value: Event| {
            let sender = sender.clone();
            let query_spawned = query_spawned.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let is_dark_mode = query_spawned.borrow().matches();

                if let Err(error) = sender.send(is_dark_mode).await {
                    web_sys::console::error_2(
                        &"Dark mode listener error: ".into(),
                        &error.to_string().into(),
                    );
                }
            });
        }) as Box<dyn Fn(_)>);

        if let Ok(callback_fn) = callback.as_ref().clone().dyn_into::<js_sys::Function>() {
            wasm_bindgen_futures::spawn_local(async move {
                query.borrow_mut().set_onchange(Some(&callback_fn));
            });
            callback.forget();

            Ok(receiver)
        } else {
            web_sys::console::error_1(
                        &"Unable to set `onchange` event lister for event checking when a user switches from dark mode to light mode and vise-versa!".into(),
                    );

            Err(WasmToolkitError::AddEventListener(
                "`onchange` event listener for checking dark and light modes".to_string(),
            ))
        }
    }
}
