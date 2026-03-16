use core::fmt;

use async_channel::Sender;
use wasm_bindgen::{JsCast, JsError, JsValue, prelude::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::DragEvent;

use crate::{NotificationType, WasmDocument, WasmToolkitError, WasmToolkitResult};

impl WasmDocument {
    pub fn configure_dropzone(
        &self,
        element_id: &str,
        sender: async_channel::Sender<FileUploadInfo>,
        error_sender: Option<async_channel::Sender<NotificationType>>,
        mime_filters: &[&str],
    ) -> WasmToolkitResult<()> {
        let input = self
            .get_element_by_id(element_id)
            .ok_or(WasmToolkitError::MissingDocumentElement)?;

        let dropzone = input
            .closest("label")
            .map_err(|error| {
                WasmToolkitError::parse_js_error(
                    error,
                    "Error trying to find closest dropzone element",
                )
            })?
            .ok_or(WasmToolkitError::UnableToFindClosedElement(format!(
                "Fallback error: Dropzone label with element id: {element_id}"
            )))?;

        {
            let cloned_dropzone = dropzone.clone();
            let error_sender1 = error_sender.clone();

            let drag_over_event_callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
                event.prevent_default();
                if let Err(error) = cloned_dropzone.class_list().add_1("hover") {
                    let formatted_error = WasmToolkitError::parse_js_error(
                        error.clone(),
                        "Error trying to add the class `hover` to dropzone element",
                    );

                    web_sys::console::error_1(&formatted_error.to_string().into());

                    error_sender_fn(error_sender1.clone(), error);
                }
            })
                as Box<dyn FnMut(web_sys::Event)>);

            dropzone
                .add_event_listener_with_callback(
                    "dragover",
                    drag_over_event_callback.as_ref().unchecked_ref(),
                )
                .map_err(|error| {
                    WasmToolkitError::parse_js_error(
                        error,
                        "Error trying to set drag over event listener for dropzone element",
                    )
                })?;

            drag_over_event_callback.forget();
        }

        {
            let error_sender = error_sender.clone();
            let cloned_dropzone = dropzone.clone();
            let drag_leave_event_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Err(error) = cloned_dropzone.class_list().remove_1("hover") {
                    let formatted_error = WasmToolkitError::parse_js_error(
                        error.clone(),
                        "Error trying to remove the class `hover` to dropzone element",
                    );

                    web_sys::console::error_1(&formatted_error.to_string().into());

                    error_sender_fn(error_sender.clone(), error);
                }
            })
                as Box<dyn FnMut(web_sys::Event)>);

            dropzone
                .add_event_listener_with_callback(
                    "dragleave",
                    drag_leave_event_callback.as_ref().unchecked_ref(),
                )
                .map_err(|error| {
                    WasmToolkitError::parse_js_error(
                        error,
                        "Error trying to set drag leave event listener for dropzone element",
                    )
                })?;

            drag_leave_event_callback.forget();
        }

        // TODO: This probably can be improved but it requires some work to be resuable function

        {
            let cloned_dropzone = dropzone.clone();
            let error_sender = error_sender.clone();
            let sender = sender.clone();

            let drop_event_callback = Closure::wrap(Box::new(move |event: DragEvent| {
                event.prevent_default();

                web_sys::console::log_1(&"DROPPED EVENT RECEIVED".into());

                if let Err(error) = cloned_dropzone.class_list().remove_1("hover") {
                    error_sender_fn(error_sender.clone(), error);
                }

                if let Some(data) = event.data_transfer()
                    && let Some(file_list) = data.files()
                {
                    let num_of_files = file_list.length();

                    (0..num_of_files).for_each(|index| {
                        if let Some(file) = file_list.get(index) {
                            if mime_filters.is_empty() {
                                Self::check_and_send(file, sender.clone(), error_sender.clone());
                            } else {
                                let file_type = file.type_();

                                if mime_filters
                                    .iter()
                                    .any(|filter| file_type.starts_with(filter))
                                {
                                    Self::check_and_send(
                                        file,
                                        sender.clone(),
                                        error_sender.clone(),
                                    );
                                } else {
                                    #[cfg(debug_assertions)]
                                    web_sys::console::log_1(
                                        &"Dropped file is not allowed in the filter for mime types"
                                            .into(),
                                    );
                                }
                            }
                        }
                    });
                }
            }) as Box<dyn FnMut(DragEvent)>);

            dropzone
                .add_event_listener_with_callback(
                    "drop",
                    drop_event_callback.as_ref().unchecked_ref(),
                )
                .map_err(|error| {
                    WasmToolkitError::parse_js_error(
                        error,
                        "Error trying to set drop event listener for drop element",
                    )
                })?;

            drop_event_callback.forget();
        }

        {
            let cloned_dropzone = dropzone.clone();
            let error_sender = error_sender.clone();
            let sender = sender.clone();

            let selected_event_callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
                event.prevent_default();

                web_sys::console::log_1(&"SELECTED FILE EVENT RECEIVED".into());

                if let Err(error) = cloned_dropzone.class_list().remove_1("hover") {
                    error_sender_fn(error_sender.clone(), error);
                }

                let input = match event
                    .target()
                    .unwrap()
                    .dyn_into::<web_sys::HtmlInputElement>()
                {
                    Ok(value) => value,
                    Err(error) => {
                        error_sender_fn(error_sender.clone(), error.into());

                        return;
                    }
                };

                if let Some(file_list) = input.files() {
                    let num_of_files = file_list.length();

                    (0..num_of_files).for_each(|index| {
                        if let Some(file) = file_list.get(index) {
                            if mime_filters.is_empty() {
                                Self::check_and_send(file, sender.clone(), error_sender.clone());
                            } else {
                                let file_type = file.type_();

                                if mime_filters
                                    .iter()
                                    .any(|filter| file_type.starts_with(filter))
                                {
                                    Self::check_and_send(
                                        file,
                                        sender.clone(),
                                        error_sender.clone(),
                                    );
                                } else {
                                    #[cfg(debug_assertions)]
                                    web_sys::console::log_1(
                                        &"Selected file is not allowed in the filter for mime types"
                                            .into(),
                                    );
                                }
                            }
                        }
                    });
                }
            })
                as Box<dyn FnMut(web_sys::Event)>);

            dropzone
                .add_event_listener_with_callback(
                    "change",
                    selected_event_callback.as_ref().unchecked_ref(),
                )
                .map_err(|error| {
                    WasmToolkitError::parse_js_error(
                        error,
                        "Error trying to set select event listener for input file element",
                    )
                })?;

            selected_event_callback.forget();
        }

        Ok(())
    }

    fn check_and_send(
        file: web_sys::File,
        sender: async_channel::Sender<FileUploadInfo>,
        error_sender: Option<async_channel::Sender<NotificationType>>,
    ) {
        if !sender.is_closed() {
            let data = file.bytes();
            let error_sender = error_sender.clone();
            let sender = sender.clone();

            spawn_local(async move {
                match JsFuture::from(data).await {
                    Err(error) => {
                        error_sender_fn(error_sender.clone(), error);
                    }
                    Ok(resolved) => {
                        if let Ok(array) = resolved.dyn_into::<js_sys::Uint8Array>() {
                            let data = array.to_vec();

                            if !data.is_empty() {
                                let file_name = file.name();
                                let r#type = file.type_();
                                let size = file.size() as usize;
                                let last_modified = if let Some(parsed) =
                                    f64_to_u64_checked(file.last_modified().trunc())
                                {
                                    parsed
                                } else {
                                    error_sender_fn(error_sender.clone(), JsError::new(
                                    "Unable to convert `last_modified` f64 for Drag event into a u64 timestamp"
                                ).into());

                                    return;
                                };
                                let dropzone_outcome = FileUploadInfo {
                                    name: file_name.clone(),
                                    last_modified,
                                    size,
                                    r#type,
                                    data,
                                };

                                if sender.send(dropzone_outcome).await.is_err() {
                                    web_sys::console::error_1(
                                    &format!(
                                        "Error Sender encountered an error when trying to send back the contents of `{}` in the  dropzone!",
                                        file_name
                                    )
                                    .into(),
                                );
                                }
                            }
                        } else {
                            error_sender_fn(error_sender.clone(), JsError::new("Unable to convert the resolved bytes of the dropzone to a Uint8Array").into());
                        }
                    }
                }
            });
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub struct FileUploadInfo {
    pub name: String,
    /// Represents milliseconds since Unix epoch
    pub last_modified: u64,
    pub size: usize,
    pub r#type: String,
    pub data: Vec<u8>,
}

impl fmt::Debug for FileUploadInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash = blake3::hash(&self.data).to_hex();

        f.debug_struct("FileUploadInfo")
            .field("name", &self.name.as_str())
            .field("last_modified", &self.last_modified)
            .field("size", &self.size)
            .field("type", &self.r#type)
            .field("data", &format!("Blake3({})", hash))
            .finish()
    }
}

impl FileUploadInfo {
    pub async fn drag(event: DragEvent, mime_filters: &[&str]) -> WasmToolkitResult<Vec<Self>> {
        event.prevent_default();

        let mut files = Vec::<FileUploadInfo>::default();

        if let Some(data) = event.data_transfer()
            && let Some(file_list) = data.files()
        {
            let num_of_files = file_list.length();

            for index in 0..num_of_files {
                if let Some(file) = file_list.get(index) {
                    if mime_filters.is_empty() {
                        if let Some(parsed_file) = Self::resolve_blob(file).await? {
                            files.push(parsed_file)
                        }
                    } else {
                        let file_type = file.type_();

                        if mime_filters
                            .iter()
                            .any(|filter| file_type.starts_with(filter))
                        {
                            if let Some(parsed_file) = Self::resolve_blob(file).await? {
                                files.push(parsed_file)
                            }
                        } else {
                            #[cfg(debug_assertions)]
                            web_sys::console::log_1(
                                &"Dropped file is not allowed in the filter for mime types".into(),
                            );
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    pub async fn resolve_blob(file: web_sys::File) -> WasmToolkitResult<Option<FileUploadInfo>> {
        let resolved = JsFuture::from(file.bytes()).await.map_err(|error| {
            WasmToolkitError::parse_js_error(
                error,
                "Unable to get the blob contents of the file in the dropzone",
            )
        })?;

        match resolved.dyn_into::<js_sys::Uint8Array>() {
            Ok(data) => {
                let data = data.to_vec();

                if !data.is_empty() {
                    let file_name = file.name();
                    let r#type = file.type_();
                    let size = file.size() as usize;
                    let last_modified =
                        if let Some(parsed) = f64_to_u64_checked(file.last_modified().trunc()) {
                            parsed
                        } else {
                            return Err(WasmToolkitError::Op(
                        "Unable to convert `last_modified` f64 for Drag event into a u64 timestamp"
                            .to_string(),
                    ));
                        };
                    Ok(Some(FileUploadInfo {
                        name: file_name.clone(),
                        last_modified,
                        size,
                        r#type,
                        data,
                    }))
                } else {
                    Ok(None)
                }
            }

            Err(error) => Err(WasmToolkitError::parse_js_error(
                error,
                "The data of the dropped file is not in byte form",
            )),
        }
    }
}

fn error_sender_fn(error_sender: Option<Sender<NotificationType>>, error: JsValue) {
    if let Some(sender) = error_sender {
        spawn_local(async move {
            if let Err(error) = sender
                .send(NotificationType::Failure(WasmToolkitError::parse_js_error(
                    error,
                    "Possible NOTIFICATION ERROR channel closed",
                )))
                .await
            {
                web_sys::console::error_1(
                    &format!(
                        "Error Sender encountered an error when trying to send back the error for dropzone: `{}`",
                        error
                    )
                    .into(),
                );
            }
        });
    }
}

fn f64_to_u64_checked(value: f64) -> Option<u64> {
    if value.is_finite() && value >= 0.0 && value <= u64::MAX as f64 && value.fract() == 0.0 {
        Some(value as u64)
    } else {
        None
    }
}
