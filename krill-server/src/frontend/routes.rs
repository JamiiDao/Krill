use dioxus::prelude::*;

use crate::{Configuration, Dashboard, Errors, Home, Login, NotFound, VerifySupportMail};

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/verify-support-mail")]
    VerifySupportMail {},

    #[route("/login")]
    Login {},

    #[route("/configuration")]
    Configuration {},

    #[route("/")]
    Home {},

    #[route("/dashboard")]
    Dashboard {},

    #[route("/404")]
    NotFound {},

    #[route("/errors/:message")]
    Errors { message: String },
}

pub struct RouteUtils;

impl RouteUtils {
    pub const CONFIGURATION: &str = "/configuration";
    pub const LOGIN: &str = "/login";
    pub const LOGOUT: &str = "/logout";
    pub const VERIFY_SUPPORT_MAIL: &str = "/verify-support-mail";
    pub const DASHBOARD: &str = "/dashboard";
    pub const ERRORS: &str = "/errors";
    pub const NOT_FOUND: &str = "/404";
}
