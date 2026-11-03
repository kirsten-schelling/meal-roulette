use crate::components::ui::*;
use crate::features::roll::settings::Conflict;
use dioxus::prelude::*;

/// Rules that cannot all hold, found before rolling. Offers to turn the
/// required ones into preferences so the roll can go ahead.
#[component]
pub fn ConflictAlert(conflicts: Vec<Conflict>, on_loosen: EventHandler<()>) -> Element {
    rsx! {
        Alert { tone: Tone::Amber, title: "These rules can't all be met".to_string(),
            ul { class: "mr-alert__list",
                for conflict in conflicts.iter() {
                    li { "{conflict.message}" }
                }
            }
            div { class: "mr-alert__actions",
                Button {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    onclick: move |_| on_loosen.call(()),
                    "Make these nice-to-have"
                }
            }
        }
    }
}

/// Why a roll failed. It stays until dismissed or the next roll starts.
#[component]
pub fn RollErrorAlert(message: String, on_dismiss: EventHandler<()>) -> Element {
    rsx! {
        Alert { tone: Tone::Red, title: "Couldn't roll".to_string(),
            p { class: "mr-alert__text", "{message}" }
            div { class: "mr-alert__actions",
                Button {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    onclick: move |_| on_dismiss.call(()),
                    "Dismiss"
                }
            }
        }
    }
}
