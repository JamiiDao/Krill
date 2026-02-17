use dioxus::prelude::*;
use krill_common::{SupportedLanguages, Translator};

use crate::TailwindColors;

#[component]
pub fn Configuration() -> Element {
    let blur = TailwindColors::glassmorphism();
    let svg_bg = crate::SVG_BG_URL;
    let crystals_bg = crate::CRYSTALS_IMAGE_URL;

    rsx! {
        div { class:"flex flex-col w-full",
             div {
        class: "
            relative
            h-[200px]
            w-full
            max-w-sm
            rounded-xl
            bg-cover
            bg-center
            bg-no-repeat
            overflow-hidden
        ",
        style: "background-image: url('{crystals_bg}')",

        div {
            class: "
                absolute inset-0
                bg-white/20
                backdrop-blur-xl
                border border-white/20
                shadow-2xl
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
