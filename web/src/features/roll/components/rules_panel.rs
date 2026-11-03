use crate::components::ui::*;
use crate::features::roll::components::RuleRow;
use crate::features::roll::settings::{RollSettings, RuleTarget};
use dioxus::prelude::*;

/// One thing the "Add a rule…" search offers.
#[derive(Clone, PartialEq)]
pub struct RuleChoice {
    pub label: String,
    pub target: RuleTarget,
}

/// Every rule of the roll in one list, with a single search to add more. Shows
/// an empty state until there is a rule, and a line about the rules when folded.
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
    let options: Vec<SelectOption> = choices
        .iter()
        .filter(|c| !current.has_rule(&c.target))
        .map(|c| SelectOption::new(c.target.key(), c.label.clone()))
        .collect();

    rsx! {
        Collapsible {
            title: "Rules".to_string(),
            summary: current.rules_summary(),
            open,
            ontoggle,
            if let Some(message) = load_error.clone() {
                Alert { tone: Tone::Red, title: "Couldn't load everything you can add", "{message}" }
            }
            Select {
                label: "Add a rule".to_string(),
                value: String::new(),
                placeholder: "Add a rule…".to_string(),
                searchable: true,
                options,
                onchange: {
                    let choices = choices.clone();
                    move |key: String| {
                        if let Some(choice) = choices.iter().find(|c| c.target.key() == key) {
                            settings.write().add_rule(&choice.target);
                        }
                    }
                },
            }
            if lines.is_empty() {
                div { class: "mr-rules__empty",
                    Text { muted: true,
                        "No rules, so any recipe can come up. Search above to ask for a protein, cuisine, ingredient or difficulty, like at least 2 beef dinners or at most 1 hard one."
                    }
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
                }
            }
        }
    }
}
