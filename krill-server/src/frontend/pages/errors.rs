use dioxus::prelude::*;

#[component]
pub fn Errors(message: String) -> Element {
    rsx! {
        div { class: "flex flex-col w-full min-h-screen items-center justify-center",
            div { class: "flex w-[90%] text-center items-center justify-center text-red-500",
                "Error: {message}"
            }
        }
    }
}
