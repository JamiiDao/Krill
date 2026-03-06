use dioxus::prelude::*;
use krill_common::{SupportedLanguages, Translator};

use crate::TailwindColors;

#[component]
pub fn Configuration() -> Element {
    let glass_overlay = TailwindColors::glassmorphism_overlay();
    let glass_normal = TailwindColors::glassmorphism_sky_indigo_zinc();
    let glass_hover = TailwindColors::glassmorphism_hover_yellow_yellow_stone();
    let shield_logo: Asset = asset!("/assets/krill-shield-logo.svg");

    rsx! {
        div { class:"flex flex-col min-h-screen w-full items-center justify-center text-center",
            div {class:"flex w-full justify-center items-center",
                img {class:"flex w-[30%]", src:"{shield_logo}", alt:"Krill Shield Logo" }
            }

            // div {class:"w-[80%] text-[var(--primary-color)] text-5xl font-[headingfont] font-bold",
            //     "8 Digit Pass Code"
            // }

            // div {class:"dark:text-white light:text-black w-[90%]  md:w-[80%] lg:w-[70%]",
            //     "Enter the passcode sent to the administrator's email address"
            // }

            // div {class:"",
            //     PasscodeEntry {

            //     }
            // }

            //  div {
            //     class:"
            //         relative
            //         h-[200px]
            //         w-full
            //         max-w-sm
            //         rounded-xl
            //         overflow-hidden
            //         {glass_hover}
            //         {glass_normal}
            //     ",

            //     div {
            //         class: "
            //             {glass_overlay}
            //             flex items-center justify-center
            //             text-4xl
            //             text-white
            //         ",
            //         p { "GREAT FOX" }
            //     }
            // }
         }
    }
}
