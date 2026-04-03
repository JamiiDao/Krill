use dioxus::prelude::*;

use crate::{Loader, SELECTED_LANGUAGE};

#[component]
pub fn LoadingLanguageTranslation() -> Element {
    rsx! {
        div { class: "flex flex-col w-full min-h-screen items-center justify-center",
            div { class: "flex w-full mb-1 text-md text-center lg:text-2xl font-black font-[subheadingfont] dark:text-[var(--primary-color)] items-center justify-center p-5",
                "Loading {SELECTED_LANGUAGE.read().english()} Translation"
            }
            Loader {}
        }
    }
}
