use crate::components::ui::*;
use crate::models::difficulty_label;
use dioxus::prelude::*;
use domain::roll::PreviewMeal;

/// A roll shown on the page, not saved yet.
#[derive(Clone, PartialEq)]
pub struct RolledWeek {
    pub seed: i64,
    pub meals: Vec<PreviewMeal>,
    /// The preferences the roll fell short of, already in words.
    pub unmet: Vec<String>,
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
/// again replaces them; each dinner can be swapped for another on its own.
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
) -> Element {
    let busy = rolling || keeping || rerolling.is_some();
    let count = week.meals.len();

    rsx! {
        section { class: "mr-result", aria_label: "Rolled week",
            Heading { level: 2, "Your week" }
            p { class: "mr-sr-only", role: "status", "Rolled {count} dinners." }
            ol { class: "mr-result__list",
                for (day , meal) in week.meals.iter().enumerate() {
                    li {
                        key: "{meal.recipe_id}",
                        class: "mr-result__row",
                        style: "animation-delay: {day * 70}ms;",
                        span { class: "mr-result__day", "Day {day + 1}" }
                        div { class: "mr-result__main",
                            span { class: "mr-result__title", "{meal.title}" }
                            span { class: "mr-result__meta", "{meta_line(meal)}" }
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
                    }
                }
            }
            if !week.unmet.is_empty() {
                Alert { tone: Tone::Amber, title: "Rolled, with some preferences loosened".to_string(),
                    ul { class: "mr-alert__list",
                        for sentence in week.unmet.iter() {
                            li { "{sentence}" }
                        }
                    }
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
                    disabled: busy,
                    onclick: move |_| on_again.call(()),
                    "Roll again"
                }
                Text { small: true, muted: true, "Seed {week.seed}" }
            }
        }
    }
}
