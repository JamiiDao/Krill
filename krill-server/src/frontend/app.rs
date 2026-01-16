use std::{cell::RefCell, iter::repeat_with, rc::Rc, sync::LazyLock};

use dioxus::prelude::*;
use wasm_toolkit::{NotificationType, Notifications, WasmWindow};

use crate::{NotificationComponent, Route};

const FAVICON: Asset = asset!("/assets/favicon.png");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[allow(clippy::redundant_closure)]
pub(crate) static NOTIFICATION_MANAGER: LazyLock<Notifications> =
    LazyLock::new(|| Notifications::init());

pub(crate) static WINDOW: GlobalSignal<WasmWindow> =
    Signal::global(|| WasmWindow::new().expect("Unable to get the browser window"));

#[allow(clippy::redundant_closure)]
pub(crate) static IS_DARK_MODE: GlobalSignal<Rc<RefCell<bool>>> =
    Signal::global(|| Rc::new(RefCell::new(true)));

#[allow(clippy::redundant_closure)]
pub(crate) static PITCH_BLACK: GlobalSignal<Rc<RefCell<bool>>> =
    Signal::global(|| Rc::new(RefCell::new(false)));

pub fn app() -> Element {
    use_effect(move || {
        spawn(check_dark_mode());
    });

    spawn(async move {
        let timeout = gloo_timers::callback::Interval::new(3000, move || {
            wasm_bindgen_futures::spawn_local(async move {
                let random_notification: String =
                    repeat_with(fastrand::alphanumeric).take(10).collect();

                NOTIFICATION_MANAGER
                    .send_final(NotificationType::Success(random_notification))
                    .await;
            });
        });

        timeout.forget();
    });

    spawn(dark_mode_listener());

    let body_bg = if *IS_DARK_MODE.read().borrow() {
        if *PITCH_BLACK.read().borrow() {
            "bg-[#101010]"
        } else {
            "bg-[#000000]"
        }
    } else {
        "bg-[#fcfcfc]"
    };

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div {class:"{body_bg} flex min-h-screen",
            NotificationComponent{}
            Router::<Route> {}
        }
    }
}

async fn check_dark_mode() {
    match WINDOW.read().is_dark_mode() {
        Ok(is_dark_mode) => *IS_DARK_MODE.write().borrow_mut() = is_dark_mode,
        Err(error) => {
            NOTIFICATION_MANAGER
                .send_final(NotificationType::Failure(error))
                .await
        }
    }
}

async fn dark_mode_listener() {
    match WINDOW.read().watch_dark_mode().await {
        Err(error) => {
            NOTIFICATION_MANAGER
                .send_final(NotificationType::Failure(error))
                .await
        }
        Ok(id_dark_mode_listener) => {
            while let Ok(is_dark_mode) = id_dark_mode_listener.recv().await {
                *IS_DARK_MODE.write().borrow_mut() = is_dark_mode;
            }
        }
    }
}
