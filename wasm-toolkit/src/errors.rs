use wasm_bindgen::JsCast;

pub type WasmToolkitResult<T> = Result<T, WasmToolkitError>;

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum WasmToolkitError {
    #[error("The `Window` was not found, are you running the program in a browser environment?")]
    WindowNotFound,
    #[error("The `Document` was not found, are you running the program in a browser environment?")]
    DocumentNotFound,
    #[error("The host was not found from `window.location.host`. Error: `{0}`.")]
    HostNotFound(String),
    #[error("The origin was not found from `window.location.origin`. Error: `{0}`.")]
    OriginNotFound(String),
    #[error("The hostname was not found from `window.location.hostname`. Error: `{0}`.")]
    HostnameNotFound(String),
    #[error("The protocol was not found from `window.location.protocol`. Error: `{0}`.")]
    ProtocolNotFound(String),
    #[error("The port was not found from `window.location.port`. Error: `{0}`.")]
    PortNotFound(String),
    #[error("The href was not found from `window.location.href`. Error: `{0}`.")]
    HrefNotFound(String),
    #[error("The hash was not found from `window.location.hash`. Error: `{0}`.")]
    HashNotFound(String),
    #[error("Unable to refresh the page. `window.location.reload`. Error: `{0}`.")]
    PageReload(String),
    #[error("Could not stringify JsValue to as JsString")]
    UnableToStringifyJsValue,
    #[error("The JsValue is not a valid JsString or it is valid but not UTF-8")]
    JsStringNotValid,
    #[error("Unable to get the browser language from `window.navigator.language`")]
    BrowserLanguageNotFound,
    #[error("`window.matchMedia` query is unsupported!")]
    MatchMediaQueryUnsupported,
    #[error("{0}")]
    Op(String),
    #[error("`.addEventListener` error: `{0}`")]
    AddEventListener(String),
    #[error("`window.document.documentElement` is missing!")]
    MissingDocumentElement,
    #[error("Unable to cast `window.document.documentElement` to `web_sys::HtmlElement`")]
    UnableToCastElementToHtmlElement,
    #[error("Unable to cast `{0}` to `js_sys::Error`. The value is probably not an error")]
    UnableToCastToJsErrorObject(String),
    #[error("Unable to set a CSS property")]
    UnableToSetCssProperty,
    #[error(
        "Attempted to cast a `web_sys::Element` to a `web_sys::HtmlElement` yet that element is not HTML Element class"
    )]
    ElementIsNotHtmlElement,
    #[error("Unable to focus HTML Element. Error: {0}")]
    UnableToFocusHtmlElement(String),
    #[error("Unable to remove keyboard focus from current focused element")]
    UnableToRemoveKeyboardFocus,
    #[error("{name} - {message}")]
    JsError { name: String, message: String },
    #[error("{name} - {message}")]
    JsErrorStatic {
        name: &'static str,
        message: &'static str,
    },
    #[error("Session Storage not found in the window")]
    SessionStorageNotFound,
    #[error("Local Storage not found in the window")]
    LocalStorageNotFound,
    #[error("Unable to find the closed element. Error `{0}`!")]
    UnableToFindClosedElement(String),
    #[error("Unable to get the `visualViewport`")]
    UnableToGetVisulaViewPort,
    #[error("Unable to get the window inner height")]
    UnableToWindowInnerHeight,
    #[error("Unable to get the window inner width")]
    UnableToWindowInnerWidth,
    #[error("Unable to get the window outer height")]
    UnableToWindowOuterHeight,
    #[error("Unable to get the window outer width")]
    UnableToWindowOuterWidth,
}

impl WasmToolkitError {
    pub fn parse_js_error(error: wasm_bindgen::JsValue, fallback_error: &str) -> Self {
        match error.dyn_into::<js_sys::Error>() {
            Ok(value) => {
                let error: WasmToolkitError = value.into();

                error
            }
            Err(_) => WasmToolkitError::UnableToCastToJsErrorObject(fallback_error.to_string()),
        }
    }
}

impl From<js_sys::Error> for WasmToolkitError {
    fn from(value: js_sys::Error) -> Self {
        let name = value
            .name()
            .as_string()
            .unwrap_or("Unable to get the `name` value of `js_sys::Error`.".to_string());

        let message = value
            .message()
            .as_string()
            .unwrap_or("Unable to get the `message` value of `js_sys::Error`.".to_string());

        Self::JsError { name, message }
    }
}
