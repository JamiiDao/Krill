use dioxus::prelude::*;

use crate::KRILL_GLASS;

pub fn single_card(
    child: std::result::Result<dioxus::prelude::VNode, dioxus::prelude::RenderError>,
) -> Element {
    rsx! {
        div { class: "h-screen w-full flex flex-col justify-center items-center krill-bg-dots p-2",
            div { class: "{KRILL_GLASS} py-5 flex flex-col justify-center items-center
                w-[90dvw] md:w-[60dvw] lg:w-[40dvw] xl:w-[30dvw] min-h-[60dvh] lg:min-h-[60dvh]",
                {child}
            }
        }
    }
}
