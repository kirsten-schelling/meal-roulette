//! Palette picker for the sidebar and drawer foot: one swatch per palette,
//! each painted from that palette's own tokens through `data-palette`.

use crate::hooks::use_palette::{use_palette, PaletteChoice};
use dioxus::prelude::*;

#[component]
pub fn PalettePicker() -> Element {
    let mut palette = use_palette();
    let current = palette.choice();

    rsx! {
        div { class: "mr-palette",
            div { class: "mr-palette__name", "{current.label()}" }
            div { class: "mr-palette__grid", role: "group", aria_label: "Colour palette",
                for option in PaletteChoice::ALL {
                    button {
                        key: "{option.key()}",
                        r#type: "button",
                        class: "mr-palette__opt",
                        title: "{option.label()}",
                        aria_label: "{option.label()}",
                        aria_pressed: option == current,
                        onclick: move |_| palette.set(option),
                        span { class: "mr-palette__swatch", "data-palette": "{option.key()}" }
                    }
                }
            }
        }
    }
}
