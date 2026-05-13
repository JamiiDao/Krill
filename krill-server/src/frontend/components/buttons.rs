use dioxus::prelude::*;

use crate::Colors;

//TODO remove Primary and Clear buttons but they are tied to configuration so fix that first

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

#[derive(Debug, Clone, PartialEq)]
pub struct GlassButton {
    pub disabled: bool,
    pub text_content: String,
    pub font_args: String,
    pub filled: bool,
}

impl GlassButton {
    pub fn new(text_content: &str) -> Self {
        Self::new_inner(text_content, false)
    }

    pub fn new_disabled(text_content: &str) -> Self {
        Self::new_inner(text_content, true)
    }

    pub fn new_inner(text_content: &str, disabled: bool) -> Self {
        Self {
            disabled,
            text_content: text_content.to_string(),
            font_args: "font-black font-[subheadingfont] text-sm md:text-md".to_string(),
            filled: bool::default(),
        }
    }

    pub fn set_font_args(mut self, font_args: &str) -> Self {
        self.font_args = font_args.to_string();

        self
    }

    pub fn set_filled(mut self) -> Self {
        self.filled = true;

        self
    }

    pub fn build<F>(self, mut callback: F) -> Result<VNode, RenderError>
    where
        F: FnMut() + 'static,
    {
        let colors = if self.filled {
            "bg-[var(--primary-color)]/10 hover:bg-transparent krill-dots active:bg-[var(--primary-color)]/20
             border border-[var(--primary-color)]  rounded-[1rem] transition-transform duration-500 
             hover:scale-[1.03] transition-colors"
        } else {
            "active:bg-[var(--primary-color)]/20 border border-[var(--primary-color)]  rounded-[1rem]
                transition-transform duration-500 hover:scale-[1.03] transition-colors"
        };

        rsx! {
            div { class: "{colors} w-full",
                button {
                    class: "{self.font_args} w-full",
                    class: "krill-bg-dots krill-bg-surface-container krill-backdrop-blur-glass krill-shadow-glass
                        flex items-center justify-center text-center px-2.5 py-1",
                    disabled: self.disabled,
                    onclick: move |_| { callback() },
                    {self.text_content.as_str()}
                }
            }
        }
    }
}
