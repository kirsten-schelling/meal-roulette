//! A bar at the top of the page content that offers a reload once the server runs a newer build.

use crate::components::ui::{Button, Size};
use crate::hooks::use_app_update::use_app_update;
use dioxus::prelude::*;

#[component]
pub fn UpdateBanner() -> Element {
    let update = use_app_update();
    if !update.is_ready() {
        return rsx! {};
    }
    rsx! {
        div { class: "mr-update-banner", role: "status",
            span { "A new version is ready." }
            Button { size: Size::Sm, onclick: move |_| update.reload(), "Reload" }
        }
    }
}
