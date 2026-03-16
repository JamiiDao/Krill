use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    ButtonInfo, ClearButton, PrimaryButton, ProgressStateToUiRecord, Translations, DOCUMENT,
    NOTIFICATION_MANAGER, WINDOW,
};

const PLACEHOLDER: char = '-';
const PASSCODE_PLACEHOLDER_ARRAY: [char; 8] = [PLACEHOLDER; 8];

#[component]
pub fn Passcode() -> Element {
    let mut state_data = consume_context::<Signal<ProgressStateToUiRecord>>();
    let translations = consume_context::<Signal<Translations>>();

    let mut passcode = use_signal(|| PASSCODE_PLACEHOLDER_ARRAY);
    let mut disabled_input = use_signal(|| false);
    let mut button_disabled = use_signal(|| true);
    let secondary_button_disabled = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col min-h-[50dvh] w-full justify-center items-center",
            div { class: "flex flex-col flex-1 w-full justify-center items-center  transition duration-1000 ease-in-out flex-col",
                div { class: "w-[80%] text-[var(--primary-color)] text-5xl font-[headingfont] font-black",
                    {translations.read().translate("passcode_heading")}
                }

                div { class: "dark:text-white mb-20 font-[subheadingfont] font-thin light:text-black w-[90%]  md:w-[80%] lg:w-[70%]",
                    {translations.read().translate("passcode_subheading")}
                }

                div { class: "flex w-full items-center justify-center h-[30%]",
                    div { class: "w-[80%]  md:w-[50%] lg:w-[30%] krill-grid-passcode justify-center",
                        {
                            (0..8)
                                .enumerate()
                                .map(|(index, _)| passcode_input(
                                    index,
                                    passcode,
                                    disabled_input,
                                    button_disabled,
                                ))
                        }
                    }
                }
            }

            div { class: "flex flex-wrap flex-col lg:flex-row  h-[10dvh] w-[95%] lg:w-[50%] justify-around items-start px-4 py-10 mb-10",
                div { class: "flex lg:w-[40%] justify-center items-center",
                    ClearButton {
                        info: ButtonInfo {
                            text_content: translations.read().translate("clear_passcode"),
                            disabled: secondary_button_disabled,
                            ..Default::default()
                        },
                        callback: move || {
                            *passcode.write() = PASSCODE_PLACEHOLDER_ARRAY;
                            *button_disabled.write() = true;
                            *disabled_input.write() = false;
                        },
                    }
                }
                div { class: "flex p-2 lg:w-[30%]" }
                div { class: "flex min-w-[40%] lg:w-[30%] justify-center items-center mb-5",
                    PrimaryButton {
                        info: ButtonInfo {
                            text_content: translations.read().translate("passcode_finish"),
                            disabled: button_disabled,
                            ..Default::default()
                        },
                        callback: move || {
                            let mut state = state_data.write();
                            state.set_passcode(passcode.read().iter().collect::<String>());
                            let domain_name = match WINDOW.read().hostname() {
                                Ok(value) => value,
                                Err(error) => {
                                    spawn_local(async move {
                                        NOTIFICATION_MANAGER.send_final_error(error).await
                                    });

                                    return;
                                }
                            };
                            state.set_domain(&domain_name);

                            if let Err(error) = state.transition() {
                                spawn(async move {
                                    NOTIFICATION_MANAGER.send_final_error(error).await;
                                });
                            }

                            drop(state);
                        },
                    }
                }
            }
        }
    }
}

fn passcode_input(
    index: usize,
    mut passcode: Signal<[char; 8]>,
    mut disabled: Signal<bool>,
    mut button_disabled: Signal<bool>,
) -> Element {
    let element_id = String::from("passcode-") + index.to_string().as_str();

    rsx! {
        input {
            id: element_id,
            class: "krill-grid-item-input font-[monospacefont] rounded-lg aspect-square bg-[var(--primary-color)]
                text-center text-6xl sm:text-4xl",
            disabled: *disabled.read(),
            oninput: move |e| {
                let value = e.value();
                let value = value.replace(PLACEHOLDER, "");

                let value = value.to_uppercase().chars().next().unwrap_or(PLACEHOLDER);

                if value.is_ascii_alphanumeric() {
                    passcode.write()[index] = value;
                } else {
                    passcode.write()[index] = PLACEHOLDER;
                }

                let mut autofocus_position = passcode
                    .read()
                    .iter()
                    .skip(index)
                    .position(|value| value == &PLACEHOLDER)
                    .unwrap_or(index);
                autofocus_position += index;
                if autofocus_position <= 7 {
                    DOCUMENT
                        .read()
                        .set_focus_to_html_element(
                            &(String::from("passcode-")
                                + autofocus_position.to_string().as_str()),
                        )
                        .unwrap();
                }
                if !passcode.read().iter().any(|value| value == &PLACEHOLDER) {
                    *disabled.write() = true;
                    *button_disabled.write() = false;
                }
            },
            onkeydown: move |e| {
                // handle backspace to move focus back
                if e.key() == Key::Backspace && (*passcode.read())[index] == PLACEHOLDER
                    && index > 0
                {
                    DOCUMENT
                        .read()
                        .set_focus_to_html_element(
                            &(String::from("passcode-")
                                + (index.saturating_sub(1)).to_string().as_str()),
                        )
                        .unwrap();
                }
            },
            r#type: "text",
            value: "{(*passcode.read())[index]}",
        }
    }
}
