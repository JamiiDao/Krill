use std::collections::VecDeque;

use dioxus::prelude::*;
use wasm_toolkit::NotificationType;

use crate::frontend::{SvgNotificationBell, NOTIFICATION_MANAGER};

#[component]
pub fn NotificationComponent() -> Element {
    // let events_queue = use_signal(|| VecDeque::<(String, NotificationType)>::new());

    // spawn({
    //     let mut events_queue = events_queue;
    //     async move {
    //         while let Ok(notification_received) =
    //             NOTIFICATION_MANAGER.receiver().lock().await.recv().await
    //         {
    //             let (secs, element_id) = match &notification_received {
    //                 NotificationType::Failure(value) => (
    //                     5000,
    //                     blake3::hash(value.to_string().as_bytes())
    //                         .to_hex()
    //                         .to_string(),
    //                 ),
    //                 NotificationType::Success(value) => {
    //                     (5000, blake3::hash(value.as_bytes()).to_hex().to_string())
    //                 }
    //             };

    //             events_queue
    //                 .write()
    //                 .push_back((element_id.clone(), notification_received));

    //             // schedule_removal(events_queue, secs, element_id)
    //         }
    //     }
    // });

    // let events = events_queue.read();

    rsx! {
    //         div { id: "notifications",
    //             class: "flex flex-col items-end justify-start p-2 absolute row-end",

    //             div {
    //                 class: "flex flex-col justify-start w-[40vw]",

    //                 for (id, notification) in events.iter() {
    //                     match notification {
    //                         NotificationType::Success(value) => rsx! {
    //                             SuccessNotification {
    //                                 key: "{id}",
    //                                 element_id: id.clone(),
    //                                 notification: value.clone()
    //                             }
    //                         },
    //                         NotificationType::Failure(error) => rsx! {
    // }
    //                             ErrorNotification {
    //                                 key: "{id}",
    //                                 element_id: id.clone(),
    //                                 notification: error.to_string()
    //                             }
    //                         },
    //                     }
    //                 }
    //             }
            //}
        }
}

#[component]
fn SuccessNotification(element_id: String, notification: String) -> Element {
    rsx! {
        // div{id: element_id,
        //     class:"flex flex-row max-w-[100%] max-h-[200px] px-1 py-3 mb-10 border-[#ff660055] border-1
        //     shadow-[-5px_2px_25px_rgba(20,20,20,0.5)] rounded-lg {GLASS_BG}",
        //     div {class:"flex sm:w-[15%]", SvgNotificationBell {  }}
        // }
    }
}

#[component]
fn ErrorNotification(element_id: String, notification: String) -> Element {
    rsx! {
        div{id: element_id,

        }
    }
}

fn schedule_removal(
    mut events_queue: Signal<VecDeque<(String, NotificationType)>>,
    secs: u32,
    element_id: String,
) {
    let timeout = gloo_timers::callback::Timeout::new(secs, move || {
        events_queue.write().retain(|(id, _)| id != &element_id);
    });
    timeout.forget();
}
