use dioxus::prelude::*;
use wasm_toolkit::NotificationType;

use crate::{
    frontend::{Route, NOTIFICATION_MANAGER},
    get_server_data, post_server_data, WINDOW,
};

#[allow(clippy::redundant_closure)]
#[component]
pub fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let mut text = use_signal(|| "...".to_string());

    use_resource(|| async move {
        tracing::info!("{}", WINDOW.read().language().unwrap());

        match WINDOW.read().origin() {
            Err(error) => {
                NOTIFICATION_MANAGER
                    .send_final(NotificationType::Failure(error))
                    .await;
            }
            Ok(mut origin) => {
                origin.push_str("/api/data");

                let data = reqwest::get(origin).await.unwrap().text().await.unwrap();

                tracing::info!("{data}");
            }
        }
    });

    // let data = use_server_future(|| crate::load_route())?;

    rsx! {
        Link { to: Route::Blog { id: count() }, "Go to blog" }
        div {class:"text-red-500",
            // {if let Some(value) =  data.result(){
            //     let value = value.unwrap();
            //     rsx!{div {"BRANDING DATA FOUND: {value}"}}
            // }else {
            //     rsx!{div {"BRANDING DATA NOT FOUND"}}
            // }}

            //  {if let Some(value) =  data.result(){
            //     let value = value.unwrap();
            //     tracing::info!("LOADED STATE: {value}");
            //     rsx!{div {"BRANDING DATA FOUND: {value}"}}
            // }else { //  {if let Some(value) =  data.result(){
            //     let value = value.unwrap();
            //     tracing::info!("LOADED STATE: {value}");
            //     rsx!{div {"BRANDING DATA FOUND: {value}"}}
            // }else {
            //     rsx!{div {"BRANDING DATA NOT FOUND"}}
            // }}
            //     rsx!{div {"BRANDING DATA NOT FOUND"}}
            // }}
            h1 { "High-Five counter: {count}" }
            button { onclick: move |_| count += 1, "Up high!" }
            button { onclick: move |_| count -= 1, "Down low!" }
            button {
                onclick: move |_| async move {
                    let data = get_server_data().await?;
                    tracing::info!("Client received: {}", data);
                    text.set(data.clone());
                    post_server_data(data).await?;
                    Ok(())
                },
                {
                    "Translations go here"
                }
            }
            "Server response ->: {text}"
        }
    }
}

#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        table {
            tbody {
                for _ in 0..id {
                    tr {
                        for _ in 0..id {
                            td { "hello world!" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AppError() -> Element {
    rsx! {
        div {"AppError ROUTE"}
    }
}

#[component]
pub fn Login() -> Element {
    rsx! {
        div {"LOGIN ROUTE"}
    }
}

#[component]
pub fn NotFound() -> Element {
    rsx! {
        div {"NOT FOUND"}
    }
}
