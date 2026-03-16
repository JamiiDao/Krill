use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    backend::ConfigVerificationOutcome, ClearButton, ConfigurationProgress, PrimaryButton,
    ProgressStateToUiRecord, Translations, NOTIFICATION_MANAGER, WINDOW,
};

#[component]
pub fn Verification() -> Element {
    let mut state_data = consume_context::<Signal<ProgressStateToUiRecord>>();
    let translations = consume_context::<Signal<Translations>>();

    let events = use_signal(|| Vec::<Element>::new());
    let verification_outcome = use_signal(|| VerificationOutcome::default());

    use_future(move || async move {
        let mut stream = crate::verification_stream(state_data.read().clone()).await?;

        while let Some(Ok(event)) = stream.recv().await {
            event_matcher(events, event, verification_outcome, translations);
        }

        dioxus::Ok(())
    });

    rsx! {
        div { class: "flex flex-col w-full items-center justify-center",
            h1 { class: "dark:text-[var(--primary-color)] text-4xl font-[headingfont] font-black mb-10",

                {translations.read().translate("verification_outcome")}
            }
            for display in events.read().iter().rev() {
                {display}
            }

            div { class: "mt-20 flex w-full items-center justify-around",

                match *verification_outcome.read() {
                    VerificationOutcome::Verifying => rsx! {},
                    VerificationOutcome::Success => rsx! {
                        PrimaryButton {
                            info: crate::ButtonInfo {
                                text_content: translations.read().translate("login"),
                                disabled: use_signal(|| false),
                                width: "w-[70%] max-w-[500px]".to_string(),
                                ..Default::default()
                            },
                            callback: |_| {
                                navigator().push("/email-verification");
                            },
                        }
                    },
                    VerificationOutcome::Failure => {
                        rsx! {
                            ClearButton {
                                info: crate::ButtonInfo {
                                    text_content: translations.read().translate("check_secrets"),
                                    disabled: use_signal(|| false),
                                    width: "w-full max-w-[200px]".to_string(),
                                    ..Default::default()
                                },
                                callback: move |_| {
                                    state_data.write().set_progress_state(ConfigurationProgress::ApiSecrets);
                                },
                            }

                            PrimaryButton {
                                info: crate::ButtonInfo {
                                    text_content: translations.read().translate("try_again"),
                                    disabled: use_signal(|| false),
                                    width: "w-[70%] max-w-[200px]".to_string(),
                                    ..Default::default()
                                },
                                callback: |_| {
                                    if let Err(error) = WINDOW.read().reload_page() {
                                        spawn_local(async move {
                                            NOTIFICATION_MANAGER.send_final_error(error).await;
                                        });
                                    }
                                },
                            }
                        }
                    }
                }

            }
        }
    }
}

fn testing_event(text_content: &str) -> Element {
    rsx! {
        div { class: "font-[subheadingfont] dark:text-[var(--primary-color)] text-[var(--primary-color)] flex w-full lg:max-w-[60dvw] text-lg text-wrap flex-wrap px-4 py-2 mb-1 mt-1",
            span { class: "flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                img { src: crate::CHECKMARK_URL, alt: "checkmark" }
            }

            span { class: "hidden md:flex px-0.5  font-[subheadingfont]
                    font-bold font-black text-sm lg:text-lg dark:text-[var(--primary-color)]",
                {text_content}
            }
        }
    }
}

fn event_succeeded(text_content: &str) -> Element {
    rsx! {
        div { class: "font-[subheadingfont] flex w-full lg:max-w-[60dvw] text-lg text-wrap flex-wrap px-4 py-2 mb-1 mt-1",
            span { class: "flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                img { src: crate::CHECKMARK_URL, alt: "checkmark" }
            }

            span { class: "hidden md:flex px-0.5  font-[subheadingfont]
                    font-bold font-black text-sm lg:text-lg  text-green-500",
                {text_content}
            }
        }
    }
}

fn error_event(text_content: &str) -> Element {
    rsx! {
        div { class: "font-[subheadingfont] flex w-full lg:max-w-[60dvw] text-lg text-wrap flex-wrap px-4 py-2 mb-1 mt-1",
            span { class: "flex mr-1 max-w-[20px] min-w-[15px] w-[20%] border border-[var(--primary-color)] rounded-full p-[1px]",
                img { src: crate::ERROR_ICON, alt: "error" }
            }

            span { class: "hidden md:flex px-0.5  font-[subheadingfont]
                    font-bold font-black text-sm lg:text-lg  text-red-500",
                {text_content}
            }
        }
    }
}

fn event_matcher(
    mut events: Signal<Vec<Element>>,
    event: ConfigVerificationOutcome,
    mut verification_outcome: Signal<VerificationOutcome>,
    translations: Signal<Translations>,
) {
    match event {
        ConfigVerificationOutcome::Failure(error) => {
            verification_outcome.set(VerificationOutcome::Failure);
            events.write().push(error_event(&error));
        }
        ConfigVerificationOutcome::TestingSmtp => {
            events.write().push(testing_event(
                &translations.read().translate("testing_smtps"),
            ));
        }
        ConfigVerificationOutcome::TestingSmtpSuccess => events.write().push(event_succeeded(
            &translations.read().translate("testing_smtps_succeeded"),
        )),
        ConfigVerificationOutcome::TestingSmtpFailure(error) => {
            verification_outcome.set(VerificationOutcome::Failure);

            events.write().push(error_event(&format!(
                "{} `{error}`",
                &translations.read().translate("testing_smtps_error"),
            )));
        }
        ConfigVerificationOutcome::TestingApiKey => {
            events.write().push(testing_event(
                &translations.read().translate("testing_api_key"),
            ));
        }
        ConfigVerificationOutcome::TestingApiKeySuccess => {
            events.write().push(event_succeeded(
                &translations.read().translate("testing_api_key_succeeded"),
            ));
        }
        ConfigVerificationOutcome::TestingApiKeyFailure(error) => {
            verification_outcome.set(VerificationOutcome::Failure);

            events.write().push(error_event(&format!(
                "{} `{error}`",
                &translations.read().translate("testing_api_key_error"),
            )));
        }
        ConfigVerificationOutcome::CreatingOrganization => {
            events.write().push(testing_event(
                &translations.read().translate("creating_org"),
            ));
        }
        ConfigVerificationOutcome::OrganizationCreated => {
            events.write().push(event_succeeded(
                &translations.read().translate("created_org"),
            ));

            verification_outcome.set(VerificationOutcome::Success);
        }
        ConfigVerificationOutcome::OrganizationCreationFailed(error) => {
            verification_outcome.set(VerificationOutcome::Failure);

            events.write().push(error_event(&format!(
                "{} `{error}`",
                &translations.read().translate("create_org_failed")
            )));
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum VerificationOutcome {
    #[default]
    Verifying,
    Success,
    Failure,
}
