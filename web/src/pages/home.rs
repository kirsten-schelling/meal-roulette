//! The landing page is This week: the latest rolled plan.

use crate::features::plan::page::ThisWeek;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        ThisWeek {}
    }
}
