use dioxus::prelude::*;

use crate::{DOCUMENT, NOTIFICATION_MANAGER};

pub const PLACEHOLDER: char = '-';

pub const fn passcode_custom_array<const N: usize>() -> [char; N] {
    [PLACEHOLDER; N]
}

#[component]
pub fn PasscodeInput<const N: usize>(index: usize, mut passcode: Signal<[char; N]>) -> Element {
    let element_id = String::from("passcode-") + index.to_string().as_str();

    rsx! {
        input {
            id: element_id,
            class: "max-w-[40px] font-[monospacefont] rounded-lg aspect-square bg-[var(--primary-color)]
                text-center text-6xl sm:text-4xl",
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
                if autofocus_position <= passcode.read().len() {
                    if let Err(error) = DOCUMENT
                        .read()
                        .set_focus_to_html_element(
                            &(String::from("passcode-")
                                + autofocus_position.to_string().as_str()),
                        )
                    {
                        spawn(async move {
                            NOTIFICATION_MANAGER.send_final_error(error).await;
                        });
                    }
                }
            },
            onkeydown: move |e| {
                // handle backspace to move focus back
                if e.key() == Key::Backspace && (*passcode.read())[index] == PLACEHOLDER
                    && index > 0
                {
                    if let Err(error) = DOCUMENT
                        .read()
                        .set_focus_to_html_element(
                            &(String::from("passcode-")
                                + (index.saturating_sub(1)).to_string().as_str()),
                        )
                    {
                        spawn(async move {
                            NOTIFICATION_MANAGER.send_final_error(error).await;
                        });
                    }
                }

            },
            r#type: "text",
            value: "{(*passcode.read())[index]}",
        }
    }
}
