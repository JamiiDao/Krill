use dioxus::prelude::*;

use crate::TailwindColors;

#[component]
pub fn PrimaryButton(info: ButtonInfo, callback: EventHandler<()>) -> Element {
    let glass_overlay = TailwindColors::glassmorphism_overlay();
    let glass_normal = TailwindColors::glassmorphism_yellow_yellow_stone();
    let glass_hover = TailwindColors::glassmorphism_hover_sky_indigo_zinc();

    rsx! {
        div {class:"flex {info.width} justify-center items-center",
            div {
                class:if *info.disabled.read(){
                    "border border-[var(--primary-color)] dark:bg-[var(--secondary-color)] dark:bg-[var(--accent-color)]"
                }else{"{glass_hover} {glass_normal} "},
                class:"flex transition duration-300 ease-in-out relative h-[40px] w-full rounded-full overflow-hidden ",
                button {class:if *info.disabled.read(){"bg-[var(--secondary-color)] bg-[repeating-linear-gradient(45deg,var(--secondary-color),var(--secondary-color)_10px,var(--secondary-color)_10px,#1a1a1a_20px)]"}else {"{glass_overlay} "},
                    class:"overflow-hidden h-[40px]  w-full px-5 py-.5 font-[markoonefont]
                text-lg rounded-full",
                class:if *info.disabled.read(){" text-white/50"}else {" text-white"},
                    onclick:move|_|{
                        callback.call(())
                    },
                    disabled:*info.disabled.read(),
                    {info.text_content}
                }
            }
         }
    }
}

#[component]
pub fn ClearButton(info: ButtonInfo, callback: EventHandler<()>) -> Element {
    rsx! {
        div {class:"flex {info.width} justify-center items-center",
            button {class:"bg-transparent hover:bg-[var(--primary-color)] transition duration-300 ease-in-out border border-[var(--primary-color)]",
                class:"h-[40px]  w-full px-5 py-.5 font-[markoonefont] font-thin
                text-lg rounded-full",
                class:if *info.disabled.read(){" text-white/50"}else {" text-white"},
                onclick:move|_|{
                    callback.call(())
                },
                disabled:*info.disabled.read(),
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
}

impl Default for ButtonInfo {
    fn default() -> Self {
        Self {
            text_content: String::default(),
            disabled: use_signal(|| true),
            width: "w-full".to_string(),
        }
    }
}
