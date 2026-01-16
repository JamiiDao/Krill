use std::{rc::Rc, sync::Arc};

use async_channel::{Receiver, Sender};
use async_lock::Mutex;

use crate::{WasmToolkitError, WasmToolkitResult, WasmWindow};

pub type NotificationSender = Sender<NotificationType>;

pub struct Notifications {
    sender: NotificationSender,
    receiver: Arc<Mutex<Receiver<NotificationType>>>,
}

impl Notifications {
    pub fn init() -> Self {
        let (sender, receiver) = async_channel::unbounded();

        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn sender(&self) -> NotificationSender {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Arc<Mutex<Receiver<NotificationType>>> {
        self.receiver.clone()
    }

    pub async fn send(&self, notification: NotificationType) -> WasmToolkitResult<()> {
        self.sender
            .clone()
            .send(notification)
            .await
            .map_err(|error| WasmToolkitError::Op(error.to_string()))
    }

    /// Logs the error to console instead of returning it
    pub async fn send_final(&self, notification: NotificationType) {
        if let Err(error) = self.sender.clone().send(notification).await {
            web_sys::console::log_2(
                &"NOTIFICATION CHANNEL ERROR: ".into(),
                &error.to_string().into(),
            );
        }
    }

    pub fn schedule_removal(&self, secs: u32, element_id: String) {
        let element_id = Rc::new(element_id);

        let element_id = element_id.clone();

        let timeout = gloo_timers::callback::Timeout::new(secs, move || match WasmWindow::new() {
            Err(_) => {
                web_sys::console::log_1(
                    &"Unable to get the window to remove notifications from".into(),
                );
            }
            Ok(window) => match window.document() {
                Err(_) => {
                    web_sys::console::error_1(
                        &"Unable to get the document to remove notifications from".into(),
                    );
                }
                Ok(document) => {
                    if let Some(element) = document.inner().get_element_by_id(&element_id) {
                        element.remove();
                    } else {
                        web_sys::console::error_2(
                            &"Element with ID does not exist. Element ID: ".into(),
                            &element_id.as_str().into(),
                        );
                    }
                }
            },
        });
        timeout.forget();
    }

    pub fn close_channel(self) -> bool {
        self.sender.close()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NotificationType {
    Success(String),
    Failure(WasmToolkitError),
}
