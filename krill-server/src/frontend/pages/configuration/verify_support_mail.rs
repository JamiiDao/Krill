use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use krill_common::VerifyMailDetailsToUi;
use web_sys::wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    ButtonInfo, Loader, LoadingLanguageTranslation, PrimaryButton, Translations,
    NOTIFICATION_MANAGER, SELECTED_LANGUAGE, WINDOW,
};

#[component]
pub fn VerifySupportMail() -> Element {
    let krill_logo = asset!("/assets/icons/krill-shield-logo.svg");

    let mut translations = use_signal(|| Translations::default());

    let mut loading_langs = use_signal(|| true);
    let mut details = use_signal(|| Option::<VerifyMailDetailsToUi>::default());
    let mut error_watcher = use_signal(|| String::default());
    let mut can_resend = use_signal(|| false);
    let mut countdown = use_signal(|| Option::<u64>::default());

    use_effect(move || {
        spawn(async move {
            match Translations::get_translation(
                "support-mail-verification-page",
                SELECTED_LANGUAGE.read().code(),
            )
            .await
            {
                Ok(fetched_translations) => {
                    translations.set(fetched_translations);
                    loading_langs.set(false);
                }
                Err(error) => {
                    NOTIFICATION_MANAGER
                        .send_final(wasm_toolkit::NotificationType::Failure(
                            wasm_toolkit::WasmToolkitError::Op(error.to_string()),
                        ))
                        .await
                }
            };
        });
    });

    let mut response_handler = move |value: ServerFnResult<Vec<u8>>| match value {
        Ok(details_bytes) => match bitcode::decode::<VerifyMailDetailsToUi>(&details_bytes) {
            Ok(value) => {
                countdown.write().replace(value.retry.as_secs());

                details.write().replace(value);

                spawn(async move {
                    let countdown_cloned = countdown;

                    let interval = Rc::new(RefCell::new(0i32));
                    let interval_read = interval.clone();

                    let callback = Closure::wrap(Box::new(move || {
                        let read_countdown = countdown_cloned.read().as_ref().cloned();

                        if let Some(mut countdown_inner) = read_countdown {
                            if countdown_inner == 0 {
                                can_resend.set(true);
                                countdown.write().take();

                                WINDOW
                                    .read()
                                    .inner()
                                    .clear_interval_with_handle(*interval_read.clone().borrow());

                                return;
                            }

                            countdown_inner = countdown_inner.saturating_sub(1);
                            countdown.write().replace(countdown_inner);
                        }
                    }) as Box<dyn FnMut()>);

                    if let Ok(interval_inner) = WINDOW
                        .read()
                        .inner()
                        .set_interval_with_callback_and_timeout_and_arguments_0(
                            callback.as_ref().unchecked_ref(),
                            1_000,
                        )
                    {
                        *interval.clone().borrow_mut() = interval_inner;

                        callback.forget();

                        if details.read().is_none() {
                            WINDOW
                                .read()
                                .inner()
                                .clear_interval_with_handle(*interval.borrow());
                        }
                    } else {
                        error_watcher.set("Unable to set the interval for seconds".to_string());
                    }
                });
            }
            Err(error) => {
                error_watcher.set(error.to_string());
            }
        },
        Err(error) => match error {
            ServerFnError::ServerError {
                message,
                code: _,
                details: _,
            } => {
                error_watcher.set(message.to_string());
            }
            _ => {
                error_watcher.set(error.to_string());
            }
        },
    };

    use_effect(move || {
        spawn(async move { response_handler(crate::send_superuser_login_auth_link().await) });
    });

    rsx! {

        div { class: "flex flex-col w-full min-h-screen items-center justify-center p-5",

            if *loading_langs.read() {
                LoadingLanguageTranslation {}
            } else {
                div { class: "flex flex-col w-full items-center justify-center",
                    div { class: "flex max-w-[200px] lg:max-w-[300px]",
                        img { src: krill_logo, alt: "Shield Logo" }
                    }
                    div { class: "text-4xl font-[headingfont] dark:text-[var(--primary-color)] font-black",
                        {translations.read().translate("support_email_verify_header")}
                    }

                }

                if error_watcher.read().is_empty() {
                    if let Some(details_inner) = details.read().as_ref() {
                        div { class: "flex flex-col",
                            {translations.read().translate("support_email_verify_subheader")}
                            " "
                            {details_inner.obsf_mail.as_str()}

                            if let Some(count) = countdown.read().as_ref() {
                                div { class: "flex w-full items-center justify-center mt-1 font-[monospacefont] text-lg",
                                    "Retry in"

                                    span { class: "flex items-center justify-center ml-2 dark:text-[var(--primary-color)]",
                                        {count.to_string()}
                                        {translations.read().translate("seconds")}
                                    }
                                }
                            }

                            if *can_resend.read() {
                                div { class: "flex w-full items-center justify-center mt-10",

                                    PrimaryButton {
                                        info: ButtonInfo::new_enabled_and_width(
                                            &translations.read().translate("resend_verification_code"),
                                            "w-[70%] max-w-[400px]",
                                        ),
                                        callback: move |_| {
                                            details.set(Option::default());
                                            error_watcher.set(String::default());
                                            can_resend.set(false);
                                            countdown.write().take();

                                            spawn(async move {
                                                response_handler(crate::send_superuser_login_auth_link().await)
                                            });
                                        },
                                    }

                                }
                            }
                        }
                    } else {
                        div { class: "flex flex-col w-full text-center items-center justify-center",
                            Loader {
                                element: Some(rsx! {
                                    div { class: "items-center justify-center flex w-full dark:text-[var(--primary-color)] font-[headingfont] font-black text-2xl",
                                        "Fetching verification information"
                                    }
                                }),
                                width: "w-[70%] max-w-[500px]",
                            }
                        }
                    }
                } else {
                    div { class: "flex items-center justify-start font-[subheadingfont] flex w-[80%] lg:max-w-[60%] text-lg text-wrap flex-wrap px-4 py-2 mb-1 mt-1",
                        span { class: "flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                            img { src: crate::ERROR_ICON, alt: "error_icon" }
                        }

                        span { class: "hidden md:flex px-0.5  font-[subheadingfont] font-bold font-black text-lg lg:text-xl  dark:text-red-300 light:text-red-500",
                            {error_watcher.read().as_str()}
                        }
                    }
                }
            }
        }

    }
}
