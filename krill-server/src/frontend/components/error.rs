use dioxus::prelude::*;

#[component]
pub fn ErrorComponent(stringyfied: String) -> Element {
    rsx! {
        {stringyfied}
    }
}
