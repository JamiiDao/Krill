use dioxus::prelude::*;

#[component]
pub fn Loader(element: Option<Element>, width: Option<String>) -> Element {
    let width = if let Some(width) = width {
        width
    } else {
        "w-[80%]".to_string()
    };

    rsx! {
        div { class: "w-full flex flex-col items-center justify-center",
            if let Some(element) = element {
                div { class: "flex flex-col w-full items-center justify-center mt-1",
                    {element}
                }
            }
            div { class: "flex {width} items-center justify-center",
                div { class: "loader" }
            }
        }
    }
}
