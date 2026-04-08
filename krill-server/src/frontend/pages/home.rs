use dioxus::prelude::*;

#[allow(clippy::redundant_closure)]
#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "text-red-500", "WEBISTE GOES HERE" }
    }
}

#[component]
pub fn Login() -> Element {
    rsx! {
        div { "LOGIN ROUTE" }
    }
}

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { "Dashboard ROUTE" }
    }
}

#[component]
pub fn NotFound() -> Element {
    rsx! {
        div { "NOT FOUND" }
    }
}
