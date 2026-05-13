use std::{cell::RefCell, rc::Rc};

use countries_iso3166::Translation;
use dioxus::prelude::*;
use krill_common::{OrganizationInfo, SupportedIdentifiers, VerifyMailDetailsToUi};
use wasm_toolkit::WasmToolkitError;
use web_sys::wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    single_card, ErrorComponent, ErrorUtil, GlassButton, Loader, LoadingLanguageTranslation,
    OrgCacheOps, Translations, TranslationsMemInfo, KRILL_GLASS, NOTIFICATION_MANAGER, WINDOW,
};

#[component]
pub fn Login() -> Element {
    use_context_provider(|| Signal::new(TranslationsMemInfo::new()));

    let mut loading_langs = use_signal(|| true);

    let translations_info = consume_context::<Signal<TranslationsMemInfo>>();

    let view_switcher = use_signal(|| ViewSwitcher::default());
    let org_info = consume_context::<Signal<OrganizationInfo>>();

    use_future(move || async move {
        TranslationsMemInfo::fetch("login", translations_info).await;

        loading_langs.set(false);
    });

    if *loading_langs.read() {
        rsx! {
            LoadingLanguageTranslation {}
        }
    } else {
        let translations = &translations_info.read().translations;

        single_card(rsx! {
            div { class: "flex flex-col flex-1 w-full px-5 py-3 items-center justify-end",
                img {
                    class: "mb-5 max-w-[50%]",
                    src: org_info.read().logo_icon_to_css_base64().as_ref(),
                }

                div { class: "flex font-[headingfont] font-black text-2xl md:text-2xl lg:text-4xl mb-10",
                    {translations.translate("welcome_back")}
                }

            }

            match *view_switcher.read() {
                ViewSwitcher::Default => rsx! {
                    div { class: "flex flex-1 flex-col flex-wrap items-center justify-center w-full h-full px-2 gap-2 ",
                        div { class: "flex flex-col max-w-[80%] w-full mb-5 items-center justify-center",
                            {choose_view(&org_info.read().supported_identifiers, view_switcher, translations)}
                        }
                    }
                },
                ViewSwitcher::VerifySuperuser => rsx! {
                    VerifySuperuserView {}
                },
                ViewSwitcher::SetupAdminAccount => rsx! { "Setup Admin Account" },
                ViewSwitcher::MemberEmailAccess => rsx! { "Member Email Login" },
                ViewSwitcher::MemberUsernameAccess => rsx! { "Member Username Login" },
            }

        })
    }
}

fn choose_view(
    supported_identifiers: &SupportedIdentifiers,
    mut view_switcher: Signal<ViewSwitcher>,
    translations: &Translations,
) -> Element {
    let font_args_for_buttons = "font-black font-[subheadingfont] text-sm";

    if !supported_identifiers.superuser {
        rsx! {
            div { class: "flex w-full",
                {
                    GlassButton::new(&translations.translate("setup_accounts").to_uppercase())
                        .set_font_args(font_args_for_buttons)
                        .build(move || {
                            view_switcher.set(ViewSwitcher::VerifySuperuser);
                        })
                }
            }
        }
    } else if !supported_identifiers.admins {
        rsx! {
            div { class: "flex w-full",
                {
                    GlassButton::new("SET UP ADMIN ACCOUNTS")
                        .set_filled()
                        .set_font_args(font_args_for_buttons)
                        .build(move || {
                            view_switcher.set(ViewSwitcher::SetupAdminAccount);

                        })
                }
            }
        }
    } else {
        rsx! {
            div { class: "flex flex-col max-w-[80%] w-full mb-5 items-center justify-center",
                {
                    GlassButton::new("ACCESS WITH EMAIL")
                        .build(move || {
                            view_switcher.set(ViewSwitcher::MemberEmailAccess);
                        })
                }
            }
            div { class: "flex flex-wrap w-full items-center justify-around",
                div { class: "flex w-full",
                    {
                        GlassButton::new("ACCESS WITH USERNAME")
                            .set_filled()
                            .set_font_args(font_args_for_buttons)
                            .build(move || {
                                view_switcher.set(ViewSwitcher::MemberUsernameAccess);

                            })
                    }
                }
                div { class: "flex w-full",
                    {
                        GlassButton::new("SUPERUSER ACCESS")
                            .set_font_args(font_args_for_buttons)
                            .build(move || {
                                view_switcher.set(ViewSwitcher::VerifySuperuser);

                            })
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum ViewSwitcher {
    #[default]
    Default,
    VerifySuperuser,
    SetupAdminAccount,
    MemberEmailAccess,
    MemberUsernameAccess,
}

#[component]
fn VerifySuperuserView() -> Element {
    let translations_info = consume_context::<Signal<TranslationsMemInfo>>();
    let translations = &translations_info.read().translations;

    let mut details = use_signal(|| Option::<VerifyMailDetailsToUi>::default());
    let mut can_resend = use_signal(|| false);
    let mut countdown = use_signal(|| Option::<u64>::default());

    let response_handler = move |value: dioxus::Result<Vec<u8>>| async move {
        match value {
            Ok(details_bytes) => {
                if let Ok(value) = bitcode::decode::<VerifyMailDetailsToUi>(&details_bytes) {
                    countdown.write().replace(value.retry.as_secs());

                    details.write().replace(value);

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
                        NOTIFICATION_MANAGER
                            .send_final_error(WasmToolkitError::Op(
                                "Unable to set the interval for seconds".to_string(),
                            ))
                            .await;
                    }
                } else {
                    NOTIFICATION_MANAGER
                        .send_final_error(WasmToolkitError::Op(
                            "Unable to decode `VerifyMailDetailsToUi`!".to_string(),
                        ))
                        .await;
                }
            }
            Err(error) => {
                ErrorUtil::downcast_dioxus_error(error).await;
            }
        }
    };

    use_future(move || async move {
        let fetched = crate::send_superuser_login_auth_link().await;

        response_handler(fetched).await
    });

    rsx! {
        if let Some(details_inner) = details.read().as_ref() {
            div { class: "flex flex-col items-center justify-center",
                {translations.translate("support_email_verify_subheader")}
                " "
                {details_inner.obsf_mail.as_str()}

                if let Some(count) = countdown.read().as_ref() {
                    div { class: "flex w-full items-center justify-center mt-1 font-[monospacefont] text-lg",

                        {translations.translate("retry_in")}

                        span { class: "flex items-center justify-center ml-2 dark:text-[var(--primary-color)]",
                            {count.to_string()}
                            {translations.translate("seconds")}
                        }
                    }
                }

                if *can_resend.read() {
                    div { class: "flex w-full items-center justify-center mt-10",
                        {
                            GlassButton::new(&translations.translate("resend_verification_code"))
                                .build(move || {
                                    details.set(Option::default());
                                    can_resend.set(false);
                                    countdown.write().take();
                                    spawn(async move {
                                        let fetched =
                                        crate::send_superuser_login_auth_link().await;

                                        response_handler(fetched).await;
                                    });
                                })
                        }
                    }
                }
            }
        } else {
            div { class: "flex flex-col w-full text-center items-center justify-center",
                Loader {
                    element: Some(rsx! {
                        div { class: "items-center justify-center flex w-full dark:text-[var(--primary-color)] font-[headingfont] font-black text-2xl",
                            {translations.translate("fetching_verification_information")}
                        }
                    }),
                    width: "w-[70%] max-w-[500px]",
                }
            }
        }
    }
}
