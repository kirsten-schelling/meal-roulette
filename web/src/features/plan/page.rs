//! A rolled week: one row per day with the dinner the roller picked. Each
//! day can be re-rolled (the solver picks again under the plan's rules) or
//! swapped for a recipe chosen by hand. `/` (This week) shows the latest
//! active plan; `/planned-meals/:id` shows any plan, e.g. a past week opened
//! from the history. Both render the same view.

use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Alert, Badge, Button, ButtonVariant, CellAlign, Column, DataTable, Heading, Icon, IconButton,
    IconName, Size, Spinner, Text, Tone,
};
use crate::features::history::components::FavouriteStar;
use crate::features::history::format::{dinner_count, rolled_label};
use crate::features::history::hooks::use_local_offset::use_local_offset;
use crate::features::plan::components::SwapRecipeDialog;
use crate::features::plan::hooks::use_plan::{
    reroll_planned_meal, set_plan_favourite, use_get_plan,
};
use crate::features::shopping_list::hooks::use_shopping_list::ensure_shopping_list;
use crate::hooks::use_toast::use_toast;
use crate::models::{date_label, day_label, error_message};
use crate::router::Route;
use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;
use domain::planned_meal::PlannedMealDetail;
use uuid::Uuid;

#[component]
pub fn ThisWeek() -> Element {
    rsx! {
        PlanView { id: None }
    }
}

#[component]
pub fn PlanDetail(id: Uuid) -> Element {
    rsx! {
        PlanView { id: Some(id) }
    }
}

/// "5 dinners, 6 Oct 2026 to 10 Oct 2026. Seed 42."
fn plan_subtitle(detail: &MealPlanDetail) -> String {
    let first = detail
        .meals
        .first()
        .map_or(detail.plan.start_date, |m| m.scheduled_date);
    let last = detail
        .meals
        .last()
        .map_or(detail.plan.start_date, |m| m.scheduled_date);
    let seed = detail
        .plan
        .seed
        .map_or_else(String::new, |s| format!(" Seed {s}."));
    format!(
        "{}, {} to {}.{seed}",
        dinner_count(detail.meals.len() as i64),
        date_label(first),
        date_label(last)
    )
}

/// The latest active plan when `id` is `None`, otherwise that plan.
#[component]
fn PlanView(id: ReadSignal<Option<Uuid>>) -> Element {
    let mut plan = use_get_plan(id);
    let offset = use_local_offset();
    let toast = use_toast();
    let navigator = use_navigator();
    // The day whose re-roll is in flight, so its button shows a spinner.
    let mut rerolling = use_signal(|| None as Option<Uuid>);
    // The day being swapped; `Some` mounts the recipe picker.
    let mut swapping = use_signal(|| None as Option<PlannedMealDetail>);
    let mut starring = use_signal(|| false);
    let mut opening_list = use_signal(|| false);

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
                    error_message(&e),
                    Tone::Red,
                ),
            }
            rerolling.set(None);
        });
    };

    let mut toggle_favourite = move |plan_id: Uuid, on: bool| {
        if starring() {
            return;
        }
        starring.set(true);
        spawn(async move {
            match set_plan_favourite(plan_id, on).await {
                Ok(_) => plan.restart(),
                Err(e) => toast.show(
                    Some("Couldn't update the favourite".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            starring.set(false);
        });
    };

    // Leaves `opening_list` set on success: the page is navigating away.
    let mut open_shopping_list = move |plan_id: Uuid| {
        if opening_list() {
            return;
        }
        opening_list.set(true);
        spawn(async move {
            match ensure_shopping_list(plan_id).await {
                Ok(_) => {
                    navigator.push(Route::PlanShoppingList { plan_id });
                }
                Err(e) => {
                    toast.show(
                        Some("Couldn't make the shopping list".to_string()),
                        error_message(&e),
                        Tone::Red,
                    );
                    opening_list.set(false);
                }
            }
        });
    };

    let on_swapped = move |updated: PlannedMealDetail| {
        swapping.set(None);
        plan.restart();
        toast.show(
            Some("Swapped".to_string()),
            format!(
                "{} is now {}.",
                day_label(updated.scheduled_date),
                updated.title
            ),
            Tone::Green,
        );
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
                            Icon { name: IconName::Dice }
                        }
                    }
                    IconButton {
                        label: "Swap for a recipe you pick".to_string(),
                        onclick: move |_| swapping.set(Some(meal.clone())),
                        Icon { name: IconName::Replace }
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
    let subtitle = detail.map_or_else(
        || "The week the roulette rolled for you.".to_string(),
        plan_subtitle,
    );
    let shown = detail.map(|d| (d.plan.id, d.plan.is_favourite));
    let star_click = move |_: MouseEvent| {
        if let Some((plan_id, is_favourite)) = shown {
            toggle_favourite(plan_id, !is_favourite);
        }
    };
    let list_click = move |_: MouseEvent| {
        if let Some((plan_id, _)) = shown {
            open_shopping_list(plan_id);
        }
    };
    // Recipes already on the plan, so the swap picker can leave them out.
    let taken: Vec<Uuid> = detail
        .map(|d| d.meals.iter().map(|m| m.recipe_id).collect())
        .unwrap_or_default();

    rsx! {
        div { class: "mr-page",
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, if id().is_some() { "Past week" } else { "This week" } }
                    Text { muted: true, "{subtitle}" }
                    if let Some(d) = detail {
                        div { class: "mr-planmeta",
                            span { "Rolled {rolled_label(d.plan.created_at, offset())}" }
                            if d.plan.archived_at.is_some() {
                                Badge { "Archived" }
                            }
                            FavouriteStar {
                                on: d.plan.is_favourite,
                                busy: starring(),
                                onclick: star_click,
                            }
                            Link { to: Route::History {}, class: "mr-textlink", "History" }
                        }
                    }
                }
                div { class: "mr-pagehead__actions",
                    if detail.is_some() {
                        Button {
                            variant: ButtonVariant::Solid,
                            loading: opening_list(),
                            onclick: list_click,
                            Icon { name: IconName::Checklist }
                            "Shopping list"
                        }
                    }
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
                    Alert { tone: Tone::Red, title: "Couldn't load the plan", "{error_message(e)}" }
                },
                Some(Ok(None)) => rsx! {
                    div { class: "mr-empty",
                        div { class: "mr-empty__icon", Icon { name: IconName::Dice } }
                        Heading { level: 3, "No week rolled yet" }
                        Text { muted: true, "Spin the roller and your dinners land here. Earlier weeks are in History." }
                        div { class: "mr-empty__actions",
                            Link { to: Route::Roll {}, class: "mr-btn mr-btn--solid mr-btn--lg", "Roll a week" }
                            Link { to: Route::History {}, class: "mr-btn mr-btn--outline mr-btn--lg", "Open History" }
                        }
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
                    on_swapped,
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
