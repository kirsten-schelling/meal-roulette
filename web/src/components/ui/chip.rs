use crate::components::ui::{Icon, IconName};
use dioxus::prelude::*;

/// A pill-shaped filter tag. Two modes, chosen by which handler you pass:
///
/// - **Toggle** (pass `onclick`): the whole pill is a button. `active` fills it
///   with the accent color — use for a list of tags you can switch on/off.
/// - **Removable** (pass `onremove`): the pill shows a × button and fires
///   `onremove` when it's clicked — use for the "active filters" you can dismiss.
#[component]
pub fn Chip(
    label: String,
    /// Filled (accent) look. Only meaningful in toggle mode.
    #[props(default)]
    active: bool,
    /// Toggle mode: fires when the pill is clicked.
    #[props(optional)]
    onclick: Option<EventHandler<()>>,
    /// Removable mode: shows a × and fires when it's clicked.
    #[props(optional)]
    onremove: Option<EventHandler<()>>,
) -> Element {
    let cls = if active {
        "mr-chip mr-chip--active"
    } else {
        "mr-chip"
    };

    if let Some(remove) = onremove {
        // Removable pill: static label + a × button.
        rsx! {
            span { class: "{cls}",
                span { class: "mr-chip__label", "{label}" }
                button {
                    r#type: "button",
                    class: "mr-chip__x",
                    aria_label: "Remove",
                    onclick: move |_| remove.call(()),
                    Icon { name: IconName::X }
                }
            }
        }
    } else {
        // Toggle pill: the whole thing is the button.
        rsx! {
            button {
                r#type: "button",
                class: "{cls}",
                onclick: move |_| {
                    if let Some(cb) = onclick {
                        cb.call(());
                    }
                },
                "{label}"
            }
        }
    }
}
