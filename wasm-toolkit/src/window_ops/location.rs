use crate::{WasmDocument, WasmToolkitCommon, WasmToolkitError, WasmToolkitResult};

use super::WasmWindow;

impl WasmWindow {
    pub fn document(&self) -> WasmToolkitResult<WasmDocument> {
        self.inner()
            .document()
            .map(WasmDocument::new)
            .ok_or(WasmToolkitError::DocumentNotFound)
    }

    pub fn location(&self) -> web_sys::Location {
        self.inner().location()
    }

    /// The protocol property of the URL interface is a string containing the protocol
    /// or scheme of the URL, including the final ":".
    ///
    /// This property can be set to change the protocol of the URL. A ":"
    /// is appended to the provided string if not provided.
    /// The provided scheme has to be compatible with the rest of the URL to be considered valid.
    pub fn protocol(&self) -> WasmToolkitResult<String> {
        self.location().protocol().map_err(|error| {
            let outcome = WasmToolkitCommon::exception_or_stringify(&error);

            WasmToolkitError::ProtocolNotFound(outcome)
        })
    }

    /// The port property of the URL interface is a string containing the port number of the URL.
    /// If the port is the default for the protocol
    /// (80 for ws: and http:, 443 for wss: and https:, and 21 for ftp:),
    /// this property contains an empty string, "".
    ///
    /// This property can be set to change the port of the URL.
    /// If the URL has no host or its scheme is file:, then setting this property has no effect.
    /// It also silently ignores invalid port numbers.
    pub fn port(&self) -> WasmToolkitResult<String> {
        self.location().port().map_err(|error| {
            let outcome = WasmToolkitCommon::exception_or_stringify(&error);

            WasmToolkitError::PortNotFound(outcome)
        })
    }

    /// The hostname property of the URL interface is a string containing either
    /// the domain name or IP address of the URL. If the URL does not have a hostname,
    /// this property contains an empty string, "". IPv4 and IPv6 addresses are normalized,
    /// such as stripping leading zeros, and domain names are converted to IDN.
    ///
    /// This property can be set to change the hostname of the URL.
    /// If the URL's scheme is not hierarchical (which the URL standard calls "special schemes"),
    /// then it has no concept of a host and setting this property has no effect.
    ///
    /// ### Examples
    /// ```
    /// https://developer.mozilla.org/en-US/docs/Web/API/URL/hostname
    /// Yields: 'developer.mozilla.org
    ///
    /// 你好.com
    /// Yields: 'xn--6qq79v.com
    /// ```
    pub fn hostname(&self) -> WasmToolkitResult<String> {
        self.location().hostname().map_err(|error| {
            let outcome = WasmToolkitCommon::exception_or_stringify(&error);

            WasmToolkitError::HostnameNotFound(outcome)
        })
    }

    /// The host property of the URL interface is a string containing the host,
    /// which is the hostname, and then, if the port of the URL is nonempty, a ":",
    /// followed by the port of the URL. If the URL does not have a hostname,
    /// this property contains an empty string, "".
    ///
    /// This property can be set to change both the hostname and the port of the URL.
    /// If the URL's scheme is not hierarchical (which the URL standard calls "special schemes"),
    /// then it has no concept of a host and setting this property has no effect.
    ///
    /// **Note**: If the given value for the host setter lacks a port,
    /// the URL's port will not change. This can be unexpected as the host getter does
    /// return a URL-port string, so one might have assumed the setter to always "reset" both.
    ///
    /// ### Examples
    /// ```
    /// 1. https://developer.mozilla.org/en-US/docs/Web/API/URL/host
    /// "developer.mozilla.org"
    ///
    /// 2. https://developer.mozilla.org:443/en-US/docs/Web/API/URL/host
    /// developer.mozilla.org #The port number is not included because 443 is the scheme's default port
    ///
    /// 3. https://developer.mozilla.org:4097/en-US/docs/Web/API/URL/host
    /// developer.mozilla.org:4097
    /// ```
    pub fn host(&self) -> WasmToolkitResult<String> {
        self.location().host().map_err(|error| {
            let outcome = WasmToolkitCommon::exception_or_stringify(&error);

            WasmToolkitError::HostNotFound(outcome)
        })
    }

    /// The origin read-only property of the Location interface returns a string
    /// containing the Unicode serialization of the origin of the location's URL.
    /// It contains the scheme as the first part.
    ///
    /// The port is only included if it's not the default for the protocol.
    ///
    /// ### Example: 'https://developer.mozilla.org'
    pub fn origin(&self) -> WasmToolkitResult<String> {
        self.location().origin().map_err(|error| {
            let outcome = WasmToolkitCommon::exception_or_stringify(&error);

            WasmToolkitError::HostNotFound(outcome)
        })
    }

    /// The href property of the Location interface is a stringifier that returns
    /// a string containing the whole URL, and allows the href to be updated.
    /// Setting the value of href navigates to the provided URL.
    ///
    /// If you want redirection, use location.replace().
    /// The difference from setting the href property value is that when
    /// using the location.replace() method, after navigating to the given URL,
    /// the current page will not be saved in session history —
    /// meaning the user won't be able to use the back button to navigate to it.
    ///
    /// ### Example
    ///  https://developer.mozilla.org/en-US/docs/Web/API/URL/href
    /// Yields: https://developer.mozilla.org/en-US/docs/Web/API/URL/href
    /// ```
    pub fn href(&self) -> WasmToolkitResult<String> {
        self.location().href().map_err(|error| {
            let outcome = WasmToolkitCommon::exception_or_stringify(&error);

            WasmToolkitError::HrefNotFound(outcome)
        })
    }

    /// The hash property of the URL interface is a string containing a "#" followed
    /// by the fragment identifier of the URL. If the URL does not have a fragment identifier,
    /// this property contains an empty string, "".
    ///
    /// This property can be set to change
    /// the fragment identifier of the URL. When setting, a single "#" prefix is
    /// added to the provided value, if not already present.
    /// Setting it to "" removes the fragment identifier.
    ///
    /// The fragment is percent-encoded when setting but not percent-decoded when reading.
    pub fn hash(&self) -> WasmToolkitResult<String> {
        self.location().hash().map_err(|error| {
            let outcome: String = WasmToolkitCommon::exception_or_stringify(&error);

            WasmToolkitError::HashNotFound(outcome)
        })
    }

    pub fn navigator(&self) -> web_sys::Navigator {
        self.inner().navigator()
    }

    pub fn language(&self) -> WasmToolkitResult<String> {
        self.navigator()
            .language()
            .ok_or(WasmToolkitError::BrowserLanguageNotFound)
    }
}
