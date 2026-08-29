use crate::components::ui::{Badge, Tone};
use crate::models::difficulty_label;
use dioxus::prelude::*;

/// Difficulty as a tone-mapped badge: 1–2 green, 3 amber, 4+ red.
#[component]
pub fn DifficultyBadge(difficulty: i16) -> Element {
    let tone = match difficulty {
        1 | 2 => Tone::Green,
        3 => Tone::Amber,
        _ => Tone::Red,
    };
    rsx! {
        Badge { tone, "{difficulty_label(difficulty.into())}" }
    }
}
