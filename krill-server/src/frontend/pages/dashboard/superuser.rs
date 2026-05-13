use dioxus::prelude::*;
use krill_common::OrganizationInfo;

use super::DashboardShell;

use crate::{single_card, Translations};

pub fn superuser_dashboard(org_info: &OrganizationInfo, translations: &Translations) -> Element {
    single_card(rsx! {

        div { class: "flex flex-col flex-1 w-full px-5 py-3 items-center justify-end",
            img {
                class: "mb-5 max-w-[50%]",
                src: org_info.logo_icon_to_css_base64().as_ref(),
            }

            div { class: "flex font-[headingfont] font-black text-2xl md:text-2xl lg:text-4xl mb-10",
                {translations.translate("welcome_back")}
            }

        }
    })
}
