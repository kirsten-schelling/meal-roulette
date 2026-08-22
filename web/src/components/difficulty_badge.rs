use crate::models::difficulty_label;
use dioxus::prelude::*;

/// A small hand-built component — this is what "rolling your own" looks like.
#[component]
pub fn DifficultyBadge(difficulty: i32) -> Element {
    let bg = match difficulty {
        1 | 2 => "#2e7d32",
        3 => "#ed6c02",
        _ => "#c62828",
    };
    rsx! {
        span {
            style: "background:{bg};color:#fff;padding:0.15rem 0.55rem;border-radius:9999px;font-size:0.8rem;white-space:nowrap;",
            "{difficulty_label(difficulty)}"
        }
    }
}
