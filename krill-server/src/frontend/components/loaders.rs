use dioxus::prelude::*;

#[component]
pub fn Loader(element: Option<Element>, width: Option<String>) -> Element {
    let width = if let Some(width) = width {
        width
    } else {
        "w-[80%]".to_string()
    };

    rsx! {
        div {class: "{width} flex flex-col",
            if let Some(element) = element {
                div {class:"flex w-full items-center justify-center mt-1",
                 {element}
                }
            }
            div {class:"loader"},
        }
    }
}
