//! The rolled week: one row per day with the dinner the roller picked. Each
//! day can be re-rolled (the solver picks again under the plan's rules) or
//! swapped for a recipe chosen by hand.

use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Alert, Badge, CellAlign, Column, DataTable, Heading, IconButton, Size, Spinner, Text, Tone,
};
use crate::features::plan::components::SwapRecipeDialog;
use crate::features::plan::hooks::use_plan::{reroll_planned_meal, use_get_latest_plan};
use crate::hooks::use_toast::use_toast;
use crate::models::{date_label, day_label};
use crate::router::Route;
use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;
use domain::planned_meal::PlannedMealDetail;
use uuid::Uuid;

#[component]
pub fn PlannedMeals() -> Element {
    let mut plan = use_get_latest_plan();
    let toast = use_toast();
    // The day whose re-roll is in flight, so its button shows a spinner.
    let mut rerolling = use_signal(|| None as Option<Uuid>);
    // The day being swapped; `Some` mounts the recipe picker.
    let mut swapping = use_signal(|| None as Option<PlannedMealDetail>);

    let mut reroll = move |id: Uuid| {
        if rerolling().is_some() {
            return;
        }
        rerolling.set(Some(id));
        spawn(async move {
            match reroll_planned_meal(id).await {
                Ok(meal) => {
                    plan.restart();
                    toast.show(
                        Some("Re-rolled".to_string()),
                        format!("{} is now {}.", day_label(meal.scheduled_date), meal.title),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't re-roll".to_string()),
                    e.to_string(),
                    Tone::Red,
                ),
            }
            rerolling.set(None);
        });
    };

    let columns: Vec<Column<PlannedMealDetail>> = vec![
        Column::new("Day", |m: &PlannedMealDetail| {
            rsx! {
                span { class: "mr-cell-strong", "{day_label(m.scheduled_date)}" }
            }
        }),
        Column::new("Dinner", |m: &PlannedMealDetail| rsx! { "{m.title}" }),
        Column::new("Cuisine", |m: &PlannedMealDetail| {
            lookup_cell(m.cuisine.as_deref())
        }),
        Column::new("Protein", |m: &PlannedMealDetail| {
            lookup_cell(m.protein.as_deref())
        }),
        Column::new("Difficulty", |m: &PlannedMealDetail| {
            rsx! {
                DifficultyBadge { difficulty: m.difficulty }
            }
        }),
        Column::new("Time", |m: &PlannedMealDetail| rsx! { "{m.time_mins} min" })
            .align(CellAlign::End),
        Column::new("", move |m: &PlannedMealDetail| {
            let id = m.id;
            let meal = m.clone();
            let busy = rerolling() == Some(id);
            rsx! {
                div { class: "mr-rowactions",
                    if busy {
                        Spinner { size: Size::Sm }
                    } else {
                        IconButton {
                            label: "Re-roll this day".to_string(),
                            onclick: move |_| reroll(id),
                            "🎲"
                        }
                    }
                    IconButton {
                        label: "Swap for a recipe you pick".to_string(),
                        onclick: move |_| swapping.set(Some(meal.clone())),
                        "⇄"
                    }
                }
            }
        })
        .align(CellAlign::End),
    ];

    let loaded = plan();
    let detail: Option<&MealPlanDetail> = match &loaded {
        Some(Ok(Some(d))) => Some(d),
        _ => None,
    };
    let subtitle = match detail {
        Some(d) => {
            let first = d
                .meals
                .first()
                .map_or(d.plan.start_date, |m| m.scheduled_date);
            let last = d
                .meals
                .last()
                .map_or(d.plan.start_date, |m| m.scheduled_date);
            let seed = d
                .plan
                .seed
                .map_or_else(String::new, |s| format!(" Seed {s}."));
            format!(
                "{} dinners, {} to {}.{seed}",
                d.meals.len(),
                date_label(first),
                date_label(last)
            )
        }
        None => "The week the roulette rolled for you.".to_string(),
    };
    // Recipes already on the plan, so the swap picker can leave them out.
    let taken: Vec<Uuid> = detail
        .map(|d| d.meals.iter().map(|m| m.recipe_id).collect())
        .unwrap_or_default();

    rsx! {
        div { class: "mr-page",
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "Planned Meals" }
                    Text { muted: true, "{subtitle}" }
                }
                div { class: "mr-pagehead__actions",
                    Link {
                        to: Route::Roll {},
                        class: "mr-btn mr-btn--outline mr-btn--md",
                        "Roll a new week"
                    }
                }
            }

            match &loaded {
                None => rsx! {
                    div { class: "mr-tablepanel mr-tablestate", Spinner {} }
                },
                Some(Err(e)) => rsx! {
                    Alert { tone: Tone::Red, title: "Couldn't load the plan", "{e}" }
                },
                Some(Ok(None)) => rsx! {
                    div { class: "mr-empty",
                        div { class: "mr-empty__icon", aria_hidden: "true", "🎲" }
                        Heading { level: 3, "No week rolled yet" }
                        Text { muted: true, "Spin the roller and your dinners land here." }
                        Link { to: Route::Roll {}, class: "mr-btn mr-btn--solid mr-btn--lg", "Roll a week" }
                    }
                },
                Some(Ok(Some(d))) => rsx! {
                    div { class: "mr-tablepanel",
                        DataTable::<PlannedMealDetail> {
                            columns: columns.clone(),
                            rows: d.meals.clone(),
                            empty: "This plan has no days.".to_string(),
                        }
                    }
                },
            }

            if let Some(meal) = swapping() {
                SwapRecipeDialog {
                    meal,
                    taken: taken.clone(),
                    on_close: move |_| swapping.set(None),
                    on_swapped: move |updated: PlannedMealDetail| {
                        swapping.set(None);
                        plan.restart();
                        toast
                            .show(
                                Some("Swapped".to_string()),
                                format!(
                                    "{} is now {}.",
                                    day_label(updated.scheduled_date),
                                    updated.title,
                                ),
                                Tone::Green,
                            );
                    },
                }
            }
        }
    }
}

/// A lookup name as a badge, or a quiet "none" when the recipe has no value.
fn lookup_cell(name: Option<&str>) -> Element {
    match name {
        Some(name) => rsx! {
            Badge { "{name}" }
        },
        None => rsx! {
            span { class: "mr-text--muted", "none" }
        },
    }
}
