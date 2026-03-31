use dioxus::prelude::*;

use crate::Colors;

#[component]
pub fn PrimaryButton(info: ButtonInfo, callback: EventHandler<()>) -> Element {
    let glassmorphism = Colors::glassmorphism();

    rsx! {
        div { class: "flex {info.width} justify-center items-center",
            div {
                class: if *info.disabled.read() { "border border-[var(--primary-color)] dark:bg-[var(--accent-color)]" } else { "{glassmorphism} hover:bg-[var(--primary-color)]" },
                class: "flex transition duration-500 ease-in relative w-full rounded-full overflow-hidden ",
                button {
                    class: if *info.disabled.read() { "bg-[repeating-linear-gradient(45deg,var(--primary-color),var(--primary-color)_10px,var(--primary-color)_10px,#000000_20px)]" } else { "bg-white/10 hover:bg-[var(--primary-color)]" },
                    class: " items-center justify-center  w-full px-5 py-2 font-[subheadingfont]
                text-md rounded-full",
                    class: "{info.font_bold.as_str()}",
                    class: if *info.disabled.read() { " text-white" } else { " text-white" },
                    onclick: move |_| { callback.call(()) },
                    disabled: *info.disabled.read(),
                    {info.text_content}
                }
            }
        }
    }
}

#[component]
pub fn ClearButton(info: ButtonInfo, callback: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex {info.width} justify-center items-center",
            button {
                class: "flex bg-transparent hover:bg-[var(--primary-color)] transition duration-500 ease-in border border-[var(--primary-color)]",
                class: "items-center justify-center  w-full px-5 py-2 font-[markoonefont] font-thin
                text-md rounded-full",
                class: if *info.disabled.read() { " text-white/50" } else { " text-white" },
                onclick: move |_| { callback.call(()) },
                disabled: *info.disabled.read(),
                {info.text_content}
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonInfo {
    pub text_content: String,
    pub disabled: Signal<bool>,
    pub width: String,
    pub font_bold: String,
}

impl ButtonInfo {
    pub fn new(text_content: &str) -> Self {
        Self {
            text_content: text_content.to_string(),
            ..Default::default()
        }
    }

    pub fn new_enabled_and_width(text_content: &str, width: &str) -> Self {
        Self {
            text_content: text_content.to_string(),
            disabled: use_signal(|| false),
            width: width.to_string(),
            ..Default::default()
        }
    }
}

impl Default for ButtonInfo {
    fn default() -> Self {
        Self {
            text_content: String::default(),
            disabled: use_signal(|| true),
            width: "w-full".to_string(),
            font_bold: "text-bold".to_string(),
        }
    }
}
