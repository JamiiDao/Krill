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
    #[error("Unable to case `window.document.documentElement` to `web_sys::HtmlElement`")]
    UnableToCastElementToHtmlElement,
    #[error("Unable to set a CSS property")]
    UnableToSetCssProperty,
}
