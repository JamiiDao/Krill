use dioxus::prelude::*;
use wasm_toolkit::{FileUploadInfo, WasmToolkitCommon};

use crate::{PrimaryButton, ProgressStateToUiRecord, Translations, DOCUMENT, NOTIFICATION_MANAGER};

const ORG_NAME_PLACEHOLDER: &str = "organization_placeholder";
const SUPPORT_MAIL_PLACEHOLDER: &str = "support_mail_placeholder";

#[component]
pub fn OrgInfo() -> Element {
    let mut state_data = consume_context::<Signal<ProgressStateToUiRecord>>();
    let translations = consume_context::<Signal<Translations>>();

    let name = use_signal(|| String::default());
    let support_mail = use_signal(|| String::default());
    let logo = use_signal(|| Vec::<FileUploadInfo>::default());
    let favicon = use_signal(|| Vec::<FileUploadInfo>::default());

    rsx! {
        div { class: "flex overflow-y-auto h-full flex-col w-[90%] lg:w-90% justify-center items-center",

            OrgName { name }
            SupportMail { support_mail }

            UploadImage {
                heading: translations.read().translate("organization_logo"),
                dropped_files: logo,
                dropzone_id: "logo-dropzone-file",
            }
            UploadImage {
                heading: translations.read().translate("upload_image_heading"),
                dropped_files: favicon,
                dropzone_id: "favicon-dropzone-file",
            }

            if !name.read().is_empty()
                && name.read().as_str()
                    != translations.read().translate(ORG_NAME_PLACEHOLDER).as_str()
                && WasmToolkitCommon::is_email_valid(support_mail.read().as_str())
                && !support_mail.read().is_empty()
            {
                div { class: "flex w-full mb-10 items-center justify-center",
                    PrimaryButton {
                        info: crate::ButtonInfo::new_enabled_and_width(
                            translations.read().translate("next").as_str(),
                            "w-full max-w-[200px]",
                        ),
                        callback: move || {
                            let mut processor = state_data.write();

                            processor.set_org_name(name.read().trim());
                            processor.set_support_mail(support_mail.read().as_str());

                            if let Some(logo_info) = logo.get(0) {
                                processor.set_logo(logo_info.data.clone(), logo_info.r#type.to_string());
                            }

                            if let Some(favicon_info) = favicon.get(0) {
                                processor
                                    .set_favicon(favicon_info.data.clone(), favicon_info.r#type.to_string());
                            }
                            if let Err(error) = processor.transition() {
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
fn OrgName(name: Signal<String>) -> Element {
    let mut error = use_signal(|| false);
    let translations = consume_context::<Signal<Translations>>();

    rsx! {
        div { class: "flex flex-col w-full max-w-[500px] items-start justify-start mb-10",
            label {
                class: "w-full flex block mb-2.5 text-md font-medium text-heading font-[subheadingfont] dark:text-[var(--primary-color)]",
                r#for: "org-name",
                {translations.read().translate("organization_logo")}
            }
            div { class: "relative w-full",
                div { class: "absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none",
                    svg {
                        "aria-hidden": "true",
                        class: "w-4 lg:w-6 h-4 text-body",
                        fill: "none",
                        height: "24",
                        view_box: "0 0 24 24",
                        width: "24",
                        xmlns: "http://www.w3.org/2000/svg",
                        path {
                            d: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Zm0 0a8.949 8.949 0 0 0 4.951-1.488A3.987 3.987 0 0 0 13 16h-2a3.987 3.987 0 0 0-3.951 3.512A8.948 8.948 0 0 0 12 21Zm3-11a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z",
                            stroke: "var(--primary-color)",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                        }
                    }
                }
                input {
                    class: "block font-[monospacefont] dark:text-[var(--primary-color)] w-full ps-9 pe-3 py-2.5 bg-neutral-secondary-medium border border-[var(--primary-color)] rounded-2xl border-default-medium text-heading text-md rounded-base focus:ring-brand focus:border-brand shadow-xs placeholder:text-body",
                    id: "org-name",
                    onfocus: move |_| {
                        if name.read().as_str()
                            == translations.read().translate(ORG_NAME_PLACEHOLDER).as_str()
                        {
                            name.clear();
                        }
                    },
                    onblur: move |_| {
                        if name.read().is_empty() {
                            name.set(translations.read().translate(ORG_NAME_PLACEHOLDER));
                        }
                    },
                    oninput: move |event| {
                        let value: String = event.value();
                        let placeholder = translations.read().translate(ORG_NAME_PLACEHOLDER);

                        if value.as_str() == placeholder.as_str() || value.is_empty() {
                            error.set(true);

                        } else {
                            error.set(false);

                        }

                        name.set(value);

                    },
                    r#type: "text",
                    value: name.read().as_str(),
                    placeholder: translations.read().translate(ORG_NAME_PLACEHOLDER),
                }
            }
            div { class: "flex w-full p-1 mt-2.5 ",
                if *error.read() {
                    p { class: "text-sm text-red-400 light:text-red-900",
                        span { class: "font-medium",
                            {translations.read().translate("invalid_org_name")}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SupportMail(support_mail: Signal<String>) -> Element {
    let translations = consume_context::<Signal<Translations>>();

    rsx! {
        div { class: "flex flex-col w-full max-w-[500px] items-start justify-start mb-10",
            label {
                class: "w-full flex block mb-2.5 text-md font-medium text-heading font-[subheadingfont] dark:text-[var(--primary-color)]",
                r#for: "support-mail",
                {translations.read().translate("support_mail")}
            }
            div { class: "relative w-full",
                div { class: "absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none",
                    svg {
                        "aria_hidden": "true",
                        class: "w-4 lg:w-6 h-4 lg:w-6 text-body",
                        fill: "none",
                        height: "24",
                        view_box: "0 0 24 24",
                        width: "24",
                        xmlns: "http://www.w3.org/2000/svg",
                        path {
                            d: "m3.5 5.5 7.893 6.036a1 1 0 0 0 1.214 0L20.5 5.5M4 19h16a1 1 0 0 0 1-1V6a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1Z",
                            stroke: "var(--primary-color)",
                            stroke_linecap: "round",
                            stroke_width: "2",
                        }
                    }
                }
                input {
                    class: "block font-[monospacefont] dark:text-[var(--primary-color)] w-full ps-9 pe-3 py-2.5 bg-neutral-secondary-medium border border-[var(--primary-color)] rounded-2xl border-default-medium text-heading text-md rounded-base focus:ring-brand focus:border-brand shadow-xs placeholder:text-body",
                    id: "support-mail",
                    placeholder: translations.read().translate(SUPPORT_MAIL_PLACEHOLDER),
                    r#type: "email",
                    onfocus: move |_| {
                        if support_mail.read().as_str()
                            == translations.read().translate(SUPPORT_MAIL_PLACEHOLDER).as_str()
                        {
                            support_mail.clear();
                        }
                    },
                    onblur: move |_| {
                        if support_mail.read().is_empty() {
                            support_mail.set(translations.read().translate(SUPPORT_MAIL_PLACEHOLDER));
                        }
                    },
                    oninput: move |event| {
                        let value: String = event.value();

                        support_mail.set(value.trim().to_string());
                    },
                    value: support_mail.read().as_str(),
                }
            }

            div { class: "flex w-full p-1 mt-2.5 ",
                if !WasmToolkitCommon::is_email_valid(support_mail.read().as_str())
                    && !support_mail.read().is_empty()
                {

                    p { class: "text-sm text-red-400 light:text-red-900",
                        span { class: "font-medium",
                            {translations.read().translate("invalid_email_address")}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UploadImage(
    heading: String,
    dropped_files: Signal<Vec<FileUploadInfo>>,
    dropzone_id: String,
) -> Element {
    let translations = consume_context::<Signal<Translations>>();

    let dropzone_id_inner = dropzone_id.clone();

    use_effect(move || {
        let (sender, receiver) = async_channel::bounded::<FileUploadInfo>(10);

        if let Err(error) = DOCUMENT.read().configure_dropzone(
            dropzone_id_inner.as_str(),
            sender.clone(),
            Some(NOTIFICATION_MANAGER.sender().clone()),
            &["image/"],
        ) {
            wasm_bindgen_futures::spawn_local(async move {
                NOTIFICATION_MANAGER.send_final_error(error).await;
            });
        }

        let receiver = receiver.clone();

        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(received) = receiver.clone().recv().await {
                dropped_files.clear();
                dropped_files.push(received);
            }
        });
    });

    rsx! {
        div { class: "flex flex-col w-full max-w-[500px] items-start justify-start mb-10",
            div { class: "flex flex-col items-start justify-center w-full",
                div { class: "text-lg dark:text-[var(--primary-color)] font-[subheadingfont] mb-2",
                    {heading}
                }
                label {
                    class: "flex flex-col items-center justify-center w-full h-32 rounded-lg border-[var(--primary-color)] border-2 border-dashed border-default-strong rounded-base cursor-pointer hover:bg-[#271102]",
                    r#for: dropzone_id.as_str(),
                    div { class: "flex flex-col items-center justify-center text-body pt-5 pb-6",
                        svg {
                            "aria_hidden": "true",
                            class: "w-8 h-8 mb-4",
                            fill: "none",
                            height: "24",
                            view_box: "0 0 24 24",
                            width: "24",
                            xmlns: "http://www.w3.org/2000/svg",
                            path {
                                d: "M15 17h3a3 3 0 0 0 0-6h-.025a5.56 5.56 0 0 0 .025-.5A5.5 5.5 0 0 0 7.207 9.021C7.137 9.017 7.071 9 7 9a4 4 0 1 0 0 8h2.167M12 19v-9m0 0-2 2m2-2 2 2",
                                stroke: "var(--primary-color)",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                            }
                        }
                        p { class: "mb-2 text-sm dark:text-[var(--primary-color)]",
                            span { class: "font-semibold",
                                {translations.read().translate("click_to_upload")}
                            }
                            " "
                            {translations.read().translate("or_drag_drop")}
                        }
                        p { class: "text-xs",
                            "SVG, PNG, JPG, GIF ... ("
                            {translations.read().translate("max")}
                            ". 1MiB)"
                        }

                        {dropped_files.read().iter().map(|info| rsx! {
                            p { class: "overflow-hidden text-ellipsis dark:text-[var(--secondary-color)]", "{info.name.as_str()}" }
                        })}
                    }
                    input {
                        id: dropzone_id.as_str(),
                        class: "hidden",
                        accept: "image/*",
                        r#type: "file",
                    }
                }
            }
        }
    }
}
