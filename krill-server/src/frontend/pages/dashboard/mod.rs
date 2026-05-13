mod shell;
pub use shell::*;

mod superuser;
pub use superuser::*;

use dioxus::prelude::*;
use krill_common::{OrganizationInfo, UserRole};

use crate::{
    backend::DashboardData, ErrorComponent, Loader, LoadingLanguageTranslation, TranslationsMemInfo,
};

#[component]
pub fn Dashboard() -> Element {
    let loaded_details = use_server_future(move || crate::dashboard_data())?;

    use_context_provider(|| Signal::new(TranslationsMemInfo::new()));

    let translations_info = consume_context::<Signal<TranslationsMemInfo>>();
    let org_info = consume_context::<Signal<OrganizationInfo>>();

    let translations_loaded = use_resource(move || async move {
        TranslationsMemInfo::fetch("dashboard", translations_info).await;
    });

    match loaded_details.clone().read().as_ref() {
        None => {
            rsx! {
                Loader {}
            }
        }
        Some(Ok(dashboard_details)) => {
            if translations_loaded.read().is_none() {
                rsx! {
                    LoadingLanguageTranslation {}
                }
            } else {
                match DashboardData::decode(dashboard_details) {
                    Err(error) => rsx! {
                        ErrorComponent { stringyfied: error.to_string() }
                    },
                    Ok(decoded) => match decoded.user_role {
                        UserRole::Superuser => superuser_dashboard(
                            &org_info.read(),
                            &translations_info.read().translations,
                        ),
                        UserRole::Admin => admin_dashboard(),
                        UserRole::Member => member_dashboard(),
                    },
                }
            }
        }
        Some(Err(error)) => {
            rsx! {
                ErrorComponent {
                    stringyfied: format!(
                        "Unable to fetch the language for this page! Details: {}",
                        error.to_string(),
                    ),
                }
            }
        }
    }
}

pub fn admin_dashboard() -> Element {
    rsx! { "Admin" }
}

pub fn member_dashboard() -> Element {
    rsx! { "Member" }
}
