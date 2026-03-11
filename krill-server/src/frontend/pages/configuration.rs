use dioxus::prelude::*;
use krill_common::{SupportedLanguages, Translator};

use crate::{ButtonInfo, ClearButton, PrimaryButton, DOCUMENT};

const PLACEHOLDER: char = '-';
const PASSCODE_PLACEHOLDER_ARRAY: [char; 8] = [PLACEHOLDER; 8];

#[component]
pub fn Configuration() -> Element {
    let shield_logo: Asset = asset!("/assets/krill-shield-logo.svg");

    let mut passcode = use_signal(|| PASSCODE_PLACEHOLDER_ARRAY);
    let mut disabled_input = use_signal(|| false);
    let mut button_disabled = use_signal(|| true);
    let secondary_button_disabled = use_signal(|| false);

    rsx! {
        div { class:"flex flex-col min-h-screen w-full items-center justify-around text-center",
            div{class:"h-[85dvh] flex flex-col items-center justify-center",
                div{class:"flex flex-col w-full justify-end items-center h-[60%]",
                    div {class:"flex w-full justify-center items-center",
                        img {class:"flex w-[25%] lg:w-[15%]", src:"{shield_logo}", alt:"Krill Shield Logo" }
                    }

                    div {class:"w-[80%] text-[var(--primary-color)] text-5xl font-[bungeehairlinefont] font-black",
                        "8 Digit Pass Code"
                    }

                    div {class:"dark:text-white font-[markoonefont] font-thin light:text-black w-[90%]  md:w-[80%] lg:w-[70%]",
                        "Enter the admin passcode printed in the terminal"
                    }
                }

                div {class:"flex w-full items-center justify-center h-[30%]",
                    div {class: "w-[80%]  md:w-[50%] krill-grid-passcode justify-center",
                        {(0..8).enumerate().map(|(index, _)| passcode_input(index, passcode, disabled_input,button_disabled))}
                    }
                }
            }

            div {
                class:"flex flex-wrap flex-col lg:flex-row  h-[10dvh] w-[95%] lg:w-[50%] justify-around items-start px-4 py-10 mb-10",
                div{class:"flex lg:w-[30%] justify-center items-center",
                    ClearButton{
                        info: ButtonInfo {
                            text_content: "Clear Passcode".to_string(),
                            disabled:secondary_button_disabled,
                            ..Default::default()
                        },
                         callback: move||{
                                *passcode.write() = PASSCODE_PLACEHOLDER_ARRAY;
                                *button_disabled.write() = true;
                                *disabled_input.write() = false;
                        }
                    }
                }
                div {class:"flex p-2 lg:w-[30%]"}
                div{class:"flex min-w-[40%] lg:w-[30%] justify-center items-center mb-5",
                    PrimaryButton{
                        info: ButtonInfo {
                            text_content: "Finish".to_string(),
                            disabled:button_disabled,
                            ..Default::default()
                        },
                        callback: ||{
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
                id:element_id,
                class: "krill-grid-item-input font-[commitmonofont] rounded-lg aspect-square bg-[var(--primary-color)]
                text-center text-6xl sm:text-4xl",
                disabled:*disabled.read(),
                oninput:move|e|{
                    let value = e.value();
                    let value = value.replace(PLACEHOLDER,"");

                    let value = value.to_uppercase().chars().next().unwrap_or(PLACEHOLDER);

                    if value.is_ascii_alphanumeric(){
                        passcode.write()[index] =value;
                    }else {
                        passcode.write()[index] = PLACEHOLDER;
                    }

                    let mut autofocus_position = passcode.read().iter().skip(index).position(|value| value == &PLACEHOLDER).unwrap_or(index);
                    autofocus_position +=index;
                    if autofocus_position <= 7 {
                        DOCUMENT.read().set_focus_to_html_element(&(String::from("passcode-")+ autofocus_position.to_string().as_str())).unwrap();
                    }

                    if !passcode.read().iter().any(|value| value == &PLACEHOLDER) {
                        *disabled.write() = true;
                        *button_disabled.write() = false;
                        tracing::info!("BUTTON NOW DISABLED: {}", *button_disabled.read());
                    }
                },
                onkeydown: move |e| {
                    // handle backspace to move focus back
                    if e.key() == Key::Backspace && (*passcode.read())[index] == PLACEHOLDER && index > 0 {
                        DOCUMENT.read().set_focus_to_html_element(&(String::from("passcode-")+ (index.saturating_sub(1)).to_string().as_str())).unwrap();
                    }
                },
                type:"text",
                value:"{(*passcode.read())[index]}"
            }
    }
}
