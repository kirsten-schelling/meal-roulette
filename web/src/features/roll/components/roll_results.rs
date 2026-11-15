use crate::components::ui::*;
use crate::features::roll::components::FixButtons;
use crate::features::roll::fix::{Fix, UnmetNote};
use crate::models::difficulty_label;
use dioxus::prelude::*;
use domain::roll::PreviewMeal;
use std::collections::BTreeSet;

/// A roll shown on the page, not saved yet.
#[derive(Clone, PartialEq)]
pub struct RolledWeek {
    pub seed: i64,
    pub meals: Vec<PreviewMeal>,
    /// The preferences the roll fell short of, already in words.
    pub unmet: Vec<UnmetNote>,
    /// The days (0 is the first) whose dinner the next roll keeps.
    pub pinned: BTreeSet<usize>,
    /// A dinner was picked by hand, so `unmet` no longer describes the week.
    pub edited: bool,
}

fn meta_line(meal: &PreviewMeal) -> String {
    let mut parts = vec![
        difficulty_label(i32::from(meal.difficulty)).to_string(),
        format!("{} min", meal.time_mins),
    ];
    parts.extend(meal.cuisine.clone());
    parts.extend(meal.protein.clone());
    parts.join(" · ")
}

/// The rolled dinners, revealed in place. Keep stores them as a plan; roll
/// again replaces all but the pinned ones; each dinner can be re-rolled or
/// swapped for a recipe picked by hand, pinned or not.
#[component]
pub fn RollResults(
    week: RolledWeek,
    /// The day being re-rolled, if any.
    rerolling: Option<usize>,
    rolling: bool,
    keeping: bool,
    on_keep: EventHandler<()>,
    on_again: EventHandler<()>,
    on_reroll: EventHandler<usize>,
    on_pin: EventHandler<usize>,
    on_pick: EventHandler<usize>,
    on_fix: EventHandler<Fix>,
) -> Element {
    let busy = rolling || keeping || rerolling.is_some();
    let count = week.meals.len();
    let all_pinned = week.pinned.len() == count;

    rsx! {
        section { class: "mr-result", aria_label: "Rolled week",
            Heading { level: 2, "Your week" }
            p { class: "mr-sr-only", role: "status", "Rolled {count} dinners." }
            ol { class: "mr-result__list",
                for (day , meal) in week.meals.iter().enumerate() {
                    li {
                        key: "{meal.recipe_id}",
                        class: if week.pinned.contains(&day) { "mr-result__row mr-result__row--pinned" } else { "mr-result__row" },
                        style: "animation-delay: {day * 70}ms;",
                        span { class: "mr-result__day", "Day {day + 1}" }
                        div { class: "mr-result__main",
                            span { class: "mr-result__title", "{meal.title}" }
                            span { class: "mr-result__meta", "{meta_line(meal)}" }
                        }
                        div { class: "mr-result__tools",
                            PinButton {
                                day,
                                on: week.pinned.contains(&day),
                                disabled: busy,
                                onclick: move |_| on_pin.call(day),
                            }
                            if rerolling == Some(day) {
                                Spinner {}
                            } else {
                                button {
                                    r#type: "button",
                                    class: "mr-iconbtn",
                                    aria_label: "Roll day {day + 1} again",
                                    title: "Roll this day again",
                                    disabled: busy,
                                    onclick: move |_| on_reroll.call(day),
                                    Icon { name: IconName::Replace }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "mr-iconbtn",
                                aria_label: "Pick a recipe for day {day + 1}",
                                title: "Pick a recipe for this day",
                                disabled: busy,
                                onclick: move |_| on_pick.call(day),
                                Icon { name: IconName::Checklist }
                            }
                        }
                    }
                }
            }
            if !week.unmet.is_empty() {
                Alert { tone: Tone::Amber, title: "Rolled, with some preferences loosened".to_string(),
                    ul { class: "mr-alert__list",
                        for note in week.unmet.iter() {
                            li {
                                "{note.sentence}"
                                if let Some(fix) = note.fix.clone() {
                                    FixButtons { fixes: vec![fix], on_fix }
                                }
                            }
                        }
                    }
                    Text { small: true, muted: true,
                        "Keep the week to cook it anyway, or pick another recipe for a day."
                    }
                }
            }
            if week.edited {
                Text { small: true, muted: true,
                    "You changed a dinner, so the rules were not checked again."
                }
            }
            div { class: "mr-result__actions",
                Button {
                    variant: ButtonVariant::Solid,
                    loading: keeping,
                    disabled: rolling || rerolling.is_some(),
                    onclick: move |_| on_keep.call(()),
                    "Keep week"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: busy || all_pinned,
                    onclick: move |_| on_again.call(()),
                    if week.pinned.is_empty() {
                        "Roll again"
                    } else {
                        "Roll the rest"
                    }
                }
                Text { small: true, muted: true, "Seed {week.seed}" }
            }
        }
    }
}

/// Toggles whether roll day `day` keeps its dinner through the next roll.
#[component]
fn PinButton(day: usize, on: bool, disabled: bool, onclick: EventHandler<()>) -> Element {
    let (class, label) = if on {
        (
            "mr-iconbtn mr-pin mr-pin--on",
            format!("Unpin day {}", day + 1),
        )
    } else {
        ("mr-iconbtn mr-pin", format!("Pin day {}", day + 1))
    };
    rsx! {
        button {
            r#type: "button",
            class,
            aria_label: "{label}",
            aria_pressed: on,
            title: "{label}",
            disabled,
            onclick: move |_| onclick.call(()),
            Icon { name: IconName::Pin }
        }
    }
}
