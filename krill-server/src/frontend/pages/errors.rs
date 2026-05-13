use dioxus::prelude::*;
use krill_common::KrillError;
use wasm_toolkit::WasmToolkitError;

use crate::NOTIFICATION_MANAGER;

#[component]
pub fn Errors(message: String) -> Element {
    rsx! {
        div { class: "flex flex-col w-full min-h-screen items-center justify-center",
            div { class: "flex w-[90%] text-center items-center justify-center text-red-500",
                "Error: {message}"
            }
        }
    }
}

pub struct ErrorUtil;

impl ErrorUtil {
    pub async fn downcast_dioxus_error(error: dioxus::CapturedError) {
        if let Some(downcasted) = error.downcast_ref::<KrillError>() {
            NOTIFICATION_MANAGER
                .send_final_error(WasmToolkitError::Op(downcasted.to_string()))
                .await;
        } else {
            NOTIFICATION_MANAGER
                .send_final_error(WasmToolkitError::Op(format!(
                    "Downcast Error: {}",
                    error.to_string()
                )))
                .await;
        }
    }

    pub async fn send_final_str(error: &str) {
        NOTIFICATION_MANAGER
            .send_final_error(WasmToolkitError::Op(error.to_string()))
            .await;
    }

    pub async fn send_final_wasm_toolkit(error: WasmToolkitError) {
        NOTIFICATION_MANAGER.send_final_error(error).await;
    }
}
