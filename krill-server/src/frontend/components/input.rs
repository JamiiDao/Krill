use dioxus::prelude::*;

use crate::{TranslationsMemInfo, KRILL_GLASS};

fn svg_icon() -> Element {
    rsx! {
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
}

#[component]
pub fn GlassInput() -> Element {
    let mut error = use_signal(|| false);
    let translations_info = consume_context::<Signal<TranslationsMemInfo>>();

    let translations = &translations_info.read().translations;

    rsx! {
        div { class: "flex flex-col w-full max-w-[500px] items-center justify-centerrounded-[1rem]",
            label {
                class: "w-full flex block mb-1 text-sm font-small font-[subheadingfont] dark:text-[var(--primary-color)]",
                r#for: "org-name",
                {translations.translate("org_name")}
            }
            div { class: "relative w-full {KRILL_GLASS} flex items-center justify-center",
                div { class: "absolute inset-y-0 start-0 flex items-center ps-1 pointer-events-none",
                    {svg_icon()}
                }
                input {
                    class: "block font-[monospacefont] dark:text-[var(--primary-color)] w-full ps-9
                     pe-1.4 py-1.5 rounded-2xl border-default-medium text-heading text-md 
                     rounded-base shadow-xs placeholder:text-body",
                    id: "org-name",
                    onfocus: move |_| {}, // if name.read().as_str(),
                    onblur: move |_| {}, // if name.read().is_empty() {,
                    oninput: move |event| {}, // let value: String = event.value();,
                    r#type: "text",
                    value: "name", //,name.read().as_str(),
                    placeholder: "placeholder", // translations.read().translate(placeholder),
                }
            }
            div { class: "flex w-full p-1 mt-2.5 ",
                if *error.read() {
                    p { class: "text-sm text-red-400 light:text-red-900",
                        span { class: "font-medium",
                            // {translations.read().translate("invalid_org_name")}
                            "errorrsss"
                        }
                    }
                }
            }
        }
    }
}
