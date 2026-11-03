//! Three-way theme control for the sidebar foot: System, Light or Dark.

use crate::hooks::use_theme::{use_theme, ThemeChoice};
use dioxus::prelude::*;

#[component]
pub fn ThemeToggle() -> Element {
    let mut theme = use_theme();
    let current = theme.choice();

    rsx! {
        div { class: "mr-theme", role: "group", aria_label: "Colour theme",
            for option in ThemeChoice::ALL {
                button {
                    key: "{option.label()}",
                    r#type: "button",
                    class: "mr-theme__opt",
                    aria_pressed: option == current,
                    onclick: move |_| theme.set(option),
                    "{option.label()}"
                }
            }
        }
    }
}
