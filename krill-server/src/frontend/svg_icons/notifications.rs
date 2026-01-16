use dioxus::prelude::*;

#[component]
pub fn SvgNotificationBell() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            path {
                d: "m10 20h4a2 2 0 0 1 -4 0zm8-4v-6a6 6 0 0 0 -5-5.91v-1.09a1 1 0 0 0 -2 0v1.09a6 6 0 0 0 -5 5.91v6l-2 2h16z",
                fill: "#ff6600",
            }
        }
    }
}
