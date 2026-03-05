use dioxus::prelude::*;
use krill_common::{SupportedLanguages, Translator};

use crate::TailwindColors;

#[component]
pub fn Configuration() -> Element {
    let glass_overlay = TailwindColors::glassmorphism_overlay();
    let glass_normal = TailwindColors::glassmorphism_sky_indigo_zinc();
    let glass_hover = TailwindColors::glassmorphism_hover_yellow_yellow_stone();

    rsx! {
        div { class:"flex flex-col w-full",
             div {
                class:"
                    relative
                    h-[200px]
                    w-full
                    max-w-sm
                    rounded-xl
                    overflow-hidden
                    {glass_hover}
                    {glass_normal}
                ",

                div {
                    class: "
                        {glass_overlay}
                        flex items-center justify-center
                        text-4xl
                        text-white
                    ",
                    p { "GREAT FOX" }
                }
            }
         }
    }
}
