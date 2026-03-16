use dioxus::prelude::*;
use url::Url;

use crate::{PrimaryButton, ProgressStateToUiRecord, Translations, NOTIFICATION_MANAGER};

#[component]
pub fn ApiSecrets() -> Element {
    let mut state_data = consume_context::<Signal<ProgressStateToUiRecord>>();
    let translations = consume_context::<Signal<Translations>>();

    let apikey_defaults = "https://api.mainnet.example.com/?apikey=dummy-api-key";

    let apikey = use_signal(|| apikey_defaults.to_string());
    let username = use_signal(|| SmtpsInfo::username(&translations.read()).to_string());
    let password = use_signal(|| SmtpsInfo::password(&translations.read()).to_string());
    let smtp_server = use_signal(|| SmtpsInfo::smtp_server(&translations.read()).to_string());
    let port = use_signal(|| SmtpsInfo::port());

    let username_error_handler = use_signal(|| false);
    let password_error_handler = use_signal(|| false);
    let smtp_server_error_handler = use_signal(|| false);
    let apikey_error_handler = use_signal(|| false);

    // "username@domain.tld:password@smtp.example.com:465"
    let formatter = move |hidden: bool| -> String {
        let username = if username.read().is_empty() {
            SmtpsInfo::username(&translations.read()).to_string()
        } else {
            username.read().to_string()
        };

        let conceal_password = |current_password: &str| -> String {
            (0..current_password.len()).map(|_| '*').collect::<String>()
        };
        let password = if !password.read().is_empty() && hidden {
            conceal_password(password.read().as_str())
        } else if password.read().is_empty() && hidden {
            conceal_password(SmtpsInfo::password(&translations.read()).as_str())
        } else if !password.read().is_empty() {
            password.read().to_string()
        } else {
            SmtpsInfo::password(&translations.read())
        };
        let smtp_server = if smtp_server.read().is_empty() {
            SmtpsInfo::smtp_server(&translations.read()).to_string()
        } else {
            smtp_server.read().to_string()
        };
        format!("{}:{}@{}:{}", username, password, smtp_server, port.read())
    };

    rsx! {
        div { class: "flex flex-col w-full h-full items-center justify-around transition duration-1000 ease-in-out",

            SecretsEntry {
                input_id: "api-key",
                placeholder: apikey_defaults,
                label_text: translations.read().translate("apikey_label_text"),
                input_icon_text: translations.read().translate("api_input_icon_text"),
                error_handler: apikey_error_handler,
                error_text: translations.read().translate("apikey_invalid_error_text"),
                write_input: apikey,
                r#type: "url",
            }

            hr { class: "h-[0.2px] bg-[var(--primary-color)] border-0 flex w-full mt-5 mb-20" }

            div { class: "ransition duration-1000 ease-in-out hidden md:flex md:flex-col max-w-[90%] w-full h-full items-center justify-center mb-20",
                div { class: "flex dark:text-[var(--secondary-color)] font-black w-full mb-5 font-[headingfont] text-3xl items-center justify-center",
                    {translations.read().translate("smtp_details_heading")}
                }
                div { class: "flex w-full  items-center justify-center",
                    div { class: "flex bg-[var(--primary-color)] rounded-l-[50px] px-2 py-1.5
                        font-[monospacefont] text-md lg:text-lg text-white",
                        "smtps://"
                    }
                    div { class: "flex overflow-hidden text-ellipsis whitespace-nowrap font-[monospacefont] rounded-r-[50px] items-center justify-start border border-[var(--primary-color)] px-2 py-1.5",
                        {formatter(true)}
                    }
                }
            }

            SecretsEntry {
                input_id: "smtp-username",
                placeholder: SmtpsInfo::username(&translations.read()),
                label_text: translations.read().translate("smtp_username_label_text"),
                input_icon_text: translations.read().translate("smtp_username_icon_text"),
                error_handler: username_error_handler,
                error_text: translations.read().translate("username_invalid_error_text"),
                write_input: username,
                r#type: "text",
            }

            SecretsEntry {
                input_id: "smtp-password",
                placeholder: SmtpsInfo::password(&translations.read()),
                label_text: translations.read().translate("smtp_password_label_text"),
                input_icon_text: translations.read().translate("smtp_password_icon_text"),
                error_handler: password_error_handler,
                error_text: translations.read().translate("invalid_smtp_password_text"),
                write_input: password,
                r#type: "password",
            }

            SecretsEntry {
                input_id: "smtp-server",
                placeholder: SmtpsInfo::smtp_server(&translations.read()),
                label_text: translations.read().translate("smtp_server_label_text"),
                input_icon_text: translations.read().translate("smtp_server_icon_text"),
                error_handler: smtp_server_error_handler,
                error_text: translations.read().translate("invalid_smtp_server_error_text"),
                write_input: smtp_server,
                r#type: "text",
            }

            PortEntry { write_input: port }

            if username.read().as_str() != SmtpsInfo::username(&translations.read())
                && password.read().as_str() != SmtpsInfo::password(&translations.read())
                && smtp_server.read().as_str() != SmtpsInfo::smtp_server(&translations.read())
                && apikey.read().as_str() != apikey_defaults && !username.read().is_empty()
                && !password.read().is_empty() && !smtp_server.read().is_empty()
                && !port.read().is_empty() && !apikey.read().is_empty()
            {
                div { class: "flex w-full mb-10 items-center justify-center",
                    PrimaryButton {
                        info: crate::ButtonInfo::new_enabled_and_width("Next", "w-full max-w-[200px]"),
                        callback: move || {
                            let smtps = String::from("smtps://") + formatter(false).as_str();

                            let mut state_writer = state_data.write();

                            state_writer.set_api_key(apikey.read().as_str());
                            state_writer.set_smtps(&smtps);
                            if let Err(error) = state_writer.transition() {
                                spawn(async move {
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

#[component]
fn SecretsEntry(
    input_id: String,
    placeholder: String,
    label_text: String,
    input_icon_text: String,
    error_handler: Signal<bool>,
    error_text: String,
    write_input: Signal<String>,
    r#type: String,
) -> Element {
    let translations = consume_context::<Signal<Translations>>();

    let placeholder = use_signal(|| placeholder);
    let r#type = use_signal(|| r#type);

    rsx! {
        div { class: "flex flex-col w-full max-w-[500px] items-start justify-start mb-10 h-full",
            label {
                class: "w-full flex block mb-2.5 text-md font-medium text-heading font-[subheadingfont] dark:text-[var(--primary-color)]",
                r#for: input_id.as_str(),
                "{label_text}"
            }
            div { class: "flex w-full border border-[var(--primary-color)] rounded-[50px] border-default-medium ",
                div { class: "flex min-w-[90px] px-2.5 py-1 font-[monospacefont] items-center justify-center text-center
                text-white bg-[var(--primary-color)]  rounded-l-[50px]
                ",
                    "{input_icon_text}"
                }
                input {
                    class: "block font-[monospacefont] dark:text-[var(--primary-color)] w-full ps-9 pe-3  py-1.5
                    text-heading text-md border-transparent focus:border-transparent focus:ring-0 focus:outline-none shadow-none focus:shadow-none placeholder:text-body",
                    id: input_id.as_str(),
                    onfocus: move |_| {
                        if write_input.read().as_str() == placeholder.read().as_str() {
                            write_input.clear();
                        }
                    },
                    onblur: move |_| {
                        if write_input.read().is_empty() {
                            write_input.set(placeholder.to_string());
                        }
                    },
                    oninput: move |event| {
                        let value: String = event.value().trim().to_string();

                        if value.as_str() == placeholder.read().as_str() || value.is_empty()
                        {
                            error_handler.set(true);

                        } else {
                            error_handler.set(false);

                        }

                        if r#type.read().as_str() == "url" {
                            if Url::parse(value.as_str()).is_err() {
                                error_handler.set(true);
                            } else {
                                error_handler.set(false);
                            }
                        }

                        write_input.set(value);

                    },
                    r#type: r#type.read().as_str(),
                    value: if r#type.read().as_str() != SmtpsInfo::password(&translations.read())
    && !write_input.read().is_empty() { write_input.read().clone() },
                    placeholder: placeholder.read().as_str(),
                }
            }

            div { class: "flex w-full p-1 mt-2.5 ",
                if *error_handler.read() {
                    p { class: "text-sm text-red-400 light:text-red-900",
                        span { class: "font-medium", "{error_text}" }
                    }
                }
            }
        }

    }
}

#[component]
fn PortEntry(write_input: Signal<String>) -> Element {
    let translations = consume_context::<Signal<Translations>>();

    let placeholder = translations.read().translate("smtp_port_placeholder");
    let port_id = "port-input-id";

    let mut error_handler = use_signal(|| bool::default());

    rsx! {
        div { class: "flex flex-col w-full max-w-[500px] items-start justify-start mb-10 h-full",
            label {
                class: "w-full flex block mb-2.5 text-md font-medium text-heading font-[subheadingfont] dark:text-[var(--primary-color)]",
                r#for: port_id,
                {translations.read().translate("smtp_port_label_text")}
            }
            div { class: "flex w-full border border-[var(--primary-color)] rounded-[50px] border-default-medium ",
                div { class: "flex min-w-[90px] px-2.5 py-1 font-[monospacefont] items-center justify-center text-center
                text-white bg-[var(--primary-color)]  rounded-l-[50px]",
                    {translations.read().translate("smtp_port_icon_text")}
                }
                input {
                    class: "[appearance:textfield] block font-[monospacefont] dark:text-[var(--primary-color)] w-full ps-9 pe-3  py-1.5
                    text-heading text-md border-transparent focus:border-transparent focus:ring-0 focus:outline-none shadow-none focus:shadow-none placeholder:text-body",
                    id: port_id,
                    oninput: move |event| {
                        let value: String = event.value();

                        if let Ok(port) = value.parse::<u16>() {
                            if port < u16::MAX {
                                error_handler.set(false);

                            } else {
                                error_handler.set(true);
                            }

                        } else {
                            error_handler.set(true);
                        }

                        write_input.set(value);

                    },
                    r#type: "number",
                    value: write_input.read().clone().to_string(),
                    placeholder,
                }
            }

            div { class: "flex w-full p-1 mt-2.5 ",
                if *error_handler.read() {
                    p { class: "text-sm text-red-400 light:text-red-900",
                        span { class: "font-medium",
                            {translations.read().translate("invalid_smtp_port_error_text")}
                        }
                    }
                }
            }
        }

    }
}

struct SmtpsInfo;

impl SmtpsInfo {
    fn username(translations: &Translations) -> String {
        translations.translate("smtp_username_placeholder")
    }

    fn password(translations: &Translations) -> String {
        translations.translate("smtp_password_placeholder")
    }

    fn smtp_server(translations: &Translations) -> String {
        translations.translate("smtp_server_placeholder")
    }

    fn port() -> String {
        465.to_string()
    }
}
