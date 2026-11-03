use crate::components::ui::*;
use crate::features::roll::components::rule_row::{
    cap_band_options, cook_time_options, parse_band, RuleSentence,
};
use crate::features::roll::settings::{
    family_label, family_noun, importance_for, RollSettings, RuleTarget, DEFAULT_COOK_TIME_CAP,
};
use dioxus::prelude::*;
use domain::roll::{Bound, DifficultyBand, RuleFamily};

/// One thing a count rule can be about, offered once its type is picked.
#[derive(Clone, PartialEq)]
pub struct RuleChoice {
    pub label: String,
    pub target: RuleTarget,
}

#[derive(Clone, PartialEq)]
enum Step {
    Closed,
    Kind,
    Value(RuleFamily),
    Count {
        target: RuleTarget,
        times: u32,
        bound: Bound,
        must: bool,
    },
    CookTime(u32),
    CapDifficulty(DifficultyBand),
}

const COUNT_KINDS: [RuleFamily; 4] = [
    RuleFamily::Protein,
    RuleFamily::Cuisine,
    RuleFamily::Difficulty,
    RuleFamily::Ingredient,
];

/// "Add rule": pick a type, then its value, then how many dinners.
#[component]
pub fn RuleBuilder(settings: Signal<RollSettings>, choices: Vec<RuleChoice>) -> Element {
    let mut step = use_signal(|| Step::Closed);
    let current = settings();
    let caps = current.every_dinner;

    let body = match step() {
        Step::Closed => rsx! {
            div { class: "mr-rule-builder__add",
                Button {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    onclick: move |_| step.set(Step::Kind),
                    Icon { name: IconName::Plus }
                    "Add rule"
                }
            }
        },
        Step::Kind => rsx! {
            div { class: "mr-rule-builder__group",
                span { class: "mr-rule-builder__label", "How many dinners are" }
                div { class: "mr-rule-builder__chips",
                    for family in COUNT_KINDS {
                        Chip {
                            key: "{family:?}",
                            label: family_label(family).to_string(),
                            onclick: move |_| step.set(Step::Value(family)),
                        }
                    }
                }
            }
            if caps.max_time_mins.is_none() || caps.max_difficulty.is_none() {
                div { class: "mr-rule-builder__group",
                    span { class: "mr-rule-builder__label", "Every dinner" }
                    div { class: "mr-rule-builder__chips",
                        if caps.max_time_mins.is_none() {
                            Chip {
                                label: "Cook time".to_string(),
                                onclick: move |_| step.set(Step::CookTime(DEFAULT_COOK_TIME_CAP)),
                            }
                        }
                        if caps.max_difficulty.is_none() {
                            Chip {
                                label: "Difficulty".to_string(),
                                onclick: move |_| step.set(Step::CapDifficulty(DifficultyBand::Medium)),
                            }
                        }
                    }
                }
            }
        },
        Step::Value(family) => {
            let options: Vec<SelectOption> = choices
                .iter()
                .filter(|c| c.target.family() == family && !current.has_rule(&c.target))
                .map(|c| SelectOption::new(c.target.key(), c.label.clone()))
                .collect();
            let empty = options.is_empty();
            let choices = choices.clone();
            rsx! {
                if empty {
                    Text { small: true, muted: true,
                        "Every {family_noun(family)} already has a rule."
                    }
                } else {
                    Select {
                        label: family_label(family).to_string(),
                        value: String::new(),
                        placeholder: format!("Pick a {}…", family_noun(family)),
                        searchable: family != RuleFamily::Difficulty,
                        options,
                        onchange: move |key: String| {
                            if let Some(choice) = choices.iter().find(|c| c.target.key() == key) {
                                step.set(Step::Count {
                                    target: choice.target.clone(),
                                    times: 1,
                                    bound: Bound::AtLeast,
                                    must: false,
                                });
                            }
                        },
                    }
                }
            }
        }
        Step::Count {
            target,
            times,
            bound,
            must,
        } => {
            let editing = target.clone();
            rsx! {
                RuleSentence {
                    family: target.family(),
                    name: target.name(),
                    times,
                    bound,
                    must,
                    on_change: move |(times, bound, must)| {
                        step.set(Step::Count {
                            target: editing.clone(),
                            times,
                            bound,
                            must,
                        });
                    },
                }
                div { class: "mr-rule-builder__actions",
                    Button {
                        size: Size::Sm,
                        onclick: move |_| {
                            settings.write().add_rule(&target, times, bound, importance_for(must));
                            step.set(Step::Closed);
                        },
                        "Add"
                    }
                }
            }
        }
        Step::CookTime(minutes) => rsx! {
            div { class: "mr-rule__body",
                span { "Every dinner: cook time at most" }
                Select {
                    compact: true,
                    aria_label: "Longest cook time".to_string(),
                    value: minutes.to_string(),
                    options: cook_time_options(minutes),
                    onchange: move |value: String| {
                        if let Ok(m) = value.parse() {
                            step.set(Step::CookTime(m));
                        }
                    },
                }
            }
            div { class: "mr-rule-builder__actions",
                Button {
                    size: Size::Sm,
                    onclick: move |_| {
                        settings.write().every_dinner.max_time_mins = Some(minutes);
                        step.set(Step::Closed);
                    },
                    "Add"
                }
            }
        },
        Step::CapDifficulty(band) => rsx! {
            div { class: "mr-rule__body",
                span { "Every dinner: difficulty at most" }
                Select {
                    compact: true,
                    aria_label: "Hardest difficulty".to_string(),
                    value: format!("{band:?}"),
                    options: cap_band_options(),
                    onchange: move |value: String| {
                        if let Some(band) = parse_band(&value) {
                            step.set(Step::CapDifficulty(band));
                        }
                    },
                }
            }
            div { class: "mr-rule-builder__actions",
                Button {
                    size: Size::Sm,
                    onclick: move |_| {
                        settings.write().every_dinner.max_difficulty = Some(band);
                        step.set(Step::Closed);
                    },
                    "Add"
                }
            }
        },
    };

    if step() == Step::Closed {
        return body;
    }
    rsx! {
        div { class: "mr-rule-builder", role: "group", aria_label: "Add rule",
            div { class: "mr-rule-builder__head",
                span { class: "mr-rule-builder__title", "Add rule" }
                if step() != Step::Kind {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        onclick: move |_| step.set(Step::Kind),
                        "Back"
                    }
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    onclick: move |_| step.set(Step::Closed),
                    "Cancel"
                }
            }
            {body}
        }
    }
}
