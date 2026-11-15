use crate::components::ui::*;
use crate::features::roll::settings::{
    band_label, family_label, importance_for, rule_noun, RuleLine, COOK_TIME_CHOICES, MAX_ROLL_DAYS,
};
use dioxus::prelude::*;
use domain::roll::{Bound, DifficultyBand, Importance, RuleFamily};

pub fn bound_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("AtLeast", "At least"),
        SelectOption::new("AtMost", "At most"),
        SelectOption::new("Exactly", "Exactly"),
    ]
}

pub fn parse_bound(value: &str) -> Bound {
    match value {
        "AtMost" => Bound::AtMost,
        "Exactly" => Bound::Exactly,
        _ => Bound::AtLeast,
    }
}

/// "At most 0" and "exactly 0" mean "none of these"; "at least 0" would ask for nothing.
pub fn count_floor(bound: Bound) -> u32 {
    match bound {
        Bound::AtLeast => 1,
        Bound::AtMost | Bound::Exactly => 0,
    }
}

/// The editable sentence of a count rule: [At least] [2] Beef dinners [Must].
#[component]
pub fn RuleSentence(
    family: RuleFamily,
    name: String,
    times: u32,
    bound: Bound,
    must: bool,
    on_change: EventHandler<(u32, Bound, bool)>,
) -> Element {
    rsx! {
        div { class: "mr-rule__body",
            Select {
                compact: true,
                aria_label: format!("{name}: at least, at most or exactly"),
                value: format!("{bound:?}"),
                options: bound_options(),
                onchange: move |value: String| {
                    let bound = parse_bound(&value);
                    on_change.call((times.max(count_floor(bound)), bound, must));
                },
            }
            NumberStepper {
                value: times,
                onchange: move |n| on_change.call((n, bound, must)),
                label: format!("{name}: number of dinners"),
                min: count_floor(bound),
                max: MAX_ROLL_DAYS,
            }
            span { class: "mr-rule__noun", "{rule_noun(family, &name, times)}" }
            Switch {
                checked: must,
                label: "Must".to_string(),
                onchange: move |on| on_change.call((times, bound, on)),
            }
        }
    }
}

/// One count rule in the list, editable in place and removable.
#[component]
pub fn RuleRow(
    line: RuleLine,
    on_change: EventHandler<(u32, Bound, Importance)>,
    on_remove: EventHandler<()>,
) -> Element {
    let RuleLine {
        family,
        name,
        times,
        bound,
        importance,
        ..
    } = line;

    rsx! {
        li { class: "mr-rule",
            span { class: "mr-rule__kind", "{family_label(family)}" }
            RuleSentence {
                family,
                name: name.clone(),
                times,
                bound,
                must: importance == Importance::Required,
                on_change: move |(times, bound, must)| {
                    on_change.call((times, bound, importance_for(must)));
                },
            }
            IconButton {
                label: format!("Remove the {name} rule"),
                onclick: move |_| on_remove.call(()),
                Icon { name: IconName::X }
            }
        }
    }
}

pub fn cook_time_options(current: u32) -> Vec<SelectOption> {
    let mut minutes: Vec<u32> = COOK_TIME_CHOICES.to_vec();
    if !minutes.contains(&current) {
        minutes.push(current);
        minutes.sort_unstable();
    }
    minutes
        .into_iter()
        .map(|m| SelectOption::new(m.to_string(), format!("{m} min")))
        .collect()
}

/// The bands an "every dinner" cap can stop at; at most Hard would allow all.
pub fn cap_band_options() -> Vec<SelectOption> {
    [DifficultyBand::Easy, DifficultyBand::Medium]
        .into_iter()
        .map(|band| SelectOption::new(format!("{band:?}"), band_label(band)))
        .collect()
}

pub fn parse_band(value: &str) -> Option<DifficultyBand> {
    DifficultyBand::ALL
        .into_iter()
        .find(|band| format!("{band:?}") == value)
}

/// "Every dinner: cook time at most [45 min]", editable and removable.
#[component]
pub fn CookTimeCapRow(
    minutes: u32,
    on_change: EventHandler<u32>,
    on_remove: EventHandler<()>,
) -> Element {
    rsx! {
        li { class: "mr-rule",
            span { class: "mr-rule__kind", "Every dinner" }
            div { class: "mr-rule__body",
                span { "Cook time at most" }
                Select {
                    compact: true,
                    aria_label: "Longest cook time".to_string(),
                    value: minutes.to_string(),
                    options: cook_time_options(minutes),
                    onchange: move |value: String| {
                        if let Ok(m) = value.parse() {
                            on_change.call(m);
                        }
                    },
                }
            }
            IconButton {
                label: "Remove the cook time limit".to_string(),
                onclick: move |_| on_remove.call(()),
                Icon { name: IconName::X }
            }
        }
    }
}

/// "Every dinner: difficulty at most [Medium]", editable and removable.
#[component]
pub fn DifficultyCapRow(
    band: DifficultyBand,
    on_change: EventHandler<DifficultyBand>,
    on_remove: EventHandler<()>,
) -> Element {
    rsx! {
        li { class: "mr-rule",
            span { class: "mr-rule__kind", "Every dinner" }
            div { class: "mr-rule__body",
                span { "Difficulty at most" }
                Select {
                    compact: true,
                    aria_label: "Hardest difficulty".to_string(),
                    value: format!("{band:?}"),
                    options: cap_band_options(),
                    onchange: move |value: String| {
                        if let Some(band) = parse_band(&value) {
                            on_change.call(band);
                        }
                    },
                }
            }
            IconButton {
                label: "Remove the difficulty limit".to_string(),
                onclick: move |_| on_remove.call(()),
                Icon { name: IconName::X }
            }
        }
    }
}
