use crate::components::ui::*;
use crate::features::roll::components::FixButtons;
use crate::features::roll::fix::Fix;
use crate::features::roll::settings::Conflict;
use crate::models::error_message;
use dioxus::prelude::*;

/// Why a roll failed, and whether its rules are to blame (the server answers
/// 422 when no combination of recipes meets them) rather than the connection.
#[derive(Clone, PartialEq)]
pub struct RollFailure {
    pub message: String,
    pub rules_unmet: bool,
}

impl From<&ServerFnError> for RollFailure {
    fn from(error: &ServerFnError) -> Self {
        Self {
            message: error_message(error),
            rules_unmet: matches!(error, ServerFnError::ServerError { code: 422, .. }),
        }
    }
}

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
                    "Turn off Must for these"
                }
            }
        }
    }
}

/// Why a roll failed, with the fixes when its rules are the cause. It stays
/// until dismissed or the next roll starts.
#[component]
pub fn RollErrorAlert(
    message: String,
    fixes: Vec<Fix>,
    on_fix: EventHandler<Fix>,
    on_dismiss: EventHandler<()>,
) -> Element {
    rsx! {
        Alert { tone: Tone::Red, title: "Couldn't roll".to_string(),
            p { class: "mr-alert__text", "{message}" }
            if !fixes.is_empty() {
                FixButtons { fixes, on_fix }
            }
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
