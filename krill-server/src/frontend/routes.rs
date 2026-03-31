use dioxus::prelude::*;

use crate::{AppError, Blog, Configuration, Home, Login, LoginInit, NotFound};

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/login-init")]
    LoginInit {},

    #[route("/login")]
    Login {},

    #[route("/configuration")]
    Configuration {},

    #[route("/")]
    Home {},

    #[route("/404")]
    NotFound {},

    #[route("/404")]
    AppError {},

    #[route("/blog/:id/")]
    Blog { id: i32 },
}

pub struct RouteUtils;

impl RouteUtils {
    pub const CONFIGURATION: &str = "/configuration";
    pub const APP_ERROR: &str = "/apperror";
    pub const LOGIN: &str = "/login";
    pub const LOGIN_INIT: &str = "/login-init";
    pub const HOME: &str = "/home";
    pub const NOT_FOUND: &str = "/404";
}
