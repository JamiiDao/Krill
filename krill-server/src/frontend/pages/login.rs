use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    rsx! {
        div { class: "h-screen w-full flex flex-col justify-center items-center krill-bg-dots ",
            div { class: "krill-bg-surface-container krill-backdrop-blur-glass krill-shadow-glass
                        flex flex-col justify-center items-center w-[50dvw] h-[50dvh]",
                div { class: "", "LOGIN ROUTE" }
                div {
                    img { src: "/logo" }
                }
            }
        }
    }
}
