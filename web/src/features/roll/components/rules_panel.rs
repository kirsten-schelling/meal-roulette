use crate::components::ui::*;
use crate::features::roll::components::{
    CookTimeCapRow, DifficultyCapRow, RuleBuilder, RuleChoice, RuleRow,
};
use crate::features::roll::settings::RollSettings;
use dioxus::prelude::*;

/// Every rule of the roll in one list: the count rules, then the caps every
/// dinner follows, with one builder to add more. Shows a line about the rules
/// when folded.
#[component]
pub fn RulesPanel(
    settings: Signal<RollSettings>,
    choices: Vec<RuleChoice>,
    open: bool,
    ontoggle: EventHandler<bool>,
    #[props(optional)] load_error: Option<String>,
) -> Element {
    let current = settings();
    let lines = current.lines();
    let caps = current.every_dinner;
    let empty = lines.is_empty() && caps.count() == 0;

    rsx! {
        Collapsible {
            title: "Rules".to_string(),
            summary: current.rules_summary(),
            open,
            ontoggle,
            if let Some(message) = load_error.clone() {
                Alert { tone: Tone::Red, title: "Some options didn't load", "{message}" }
            }
            if empty {
                div { class: "mr-rules__empty",
                    Text { muted: true, "No rules yet." }
                }
            } else {
                ul { class: "mr-rules__list", aria_label: "Rules",
                    for line in lines {
                        {
                            let (family, index) = (line.family, line.index);
                            rsx! {
                                RuleRow {
                                    key: "{line.key}",
                                    line: line.clone(),
                                    on_change: move |(times, bound, importance)| {
                                        settings.write().set_rule(family, index, times, bound, importance);
                                    },
                                    on_remove: move |_| settings.write().remove_rule(family, index),
                                }
                            }
                        }
                    }
                    if let Some(minutes) = caps.max_time_mins {
                        CookTimeCapRow {
                            minutes,
                            on_change: move |m| settings.write().every_dinner.max_time_mins = Some(m),
                            on_remove: move |_| settings.write().every_dinner.max_time_mins = None,
                        }
                    }
                    if let Some(band) = caps.max_difficulty {
                        DifficultyCapRow {
                            band,
                            on_change: move |b| settings.write().every_dinner.max_difficulty = Some(b),
                            on_remove: move |_| settings.write().every_dinner.max_difficulty = None,
                        }
                    }
                }
            }
            RuleBuilder { settings, choices }
        }
    }
}
