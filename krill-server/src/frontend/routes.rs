use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{Blog, Home};

#[derive(Clone, Routable, Debug, PartialEq, Serialize, Deserialize)]
pub enum Route {
    #[route("/")]
    Home {},

    #[route("/blog/:id/")]
    Blog { id: i32 },
}
