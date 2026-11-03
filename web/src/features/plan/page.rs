//! A rolled week: one row per day with the dinner the roller picked. Each
//! day can be re-rolled (the solver picks again under the plan's rules) or
//! swapped for a recipe chosen by hand. `/planned-meals` shows the latest
//! active plan; `/planned-meals/:id` shows any plan, e.g. one opened from the
//! history. Both render the same view.

use crate::components::ui::{
    Alert, Badge, Button, ButtonVariant, Heading, Icon, IconName, Spinner, Text, Tone,
};
use crate::features::history::components::FavouriteStar;
use crate::features::history::format::{dinner_count, rolled_label};
use crate::features::history::hooks::use_local_offset::use_local_offset;
use crate::features::plan::components::{DayCard, ScheduleCard, SwapRecipeDialog};
use crate::features::plan::hooks::use_plan::{
    reroll_planned_meal, schedule_plan, schedule_planned_meal, set_plan_favourite, use_get_plan,
};
use crate::features::shopping_list::hooks::use_shopping_list::ensure_shopping_list;
use crate::hooks::use_toast::use_toast;
use crate::hooks::use_today::use_today;
use crate::models::{date_label, day_label, dinner_day_label, error_message};
use crate::router::Route;
use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;
use domain::planned_meal::PlannedMealDetail;
use time::Date;
use uuid::Uuid;

#[component]
pub fn PlannedMeals() -> Element {
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
    let dates: Vec<Date> = detail
        .meals
        .iter()
        .filter_map(|m| m.scheduled_date)
        .collect();
    let count = dinner_count(detail.meals.len() as i64);
    let span = match (dates.iter().min(), dates.iter().max()) {
        (Some(&first), Some(&last)) if dates.len() == detail.meals.len() => {
            format!(", {} to {}", date_label(first), date_label(last))
        }
        (Some(&first), Some(&last)) => format!(
            ", {} of them on days ({} to {})",
            dates.len(),
            date_label(first),
            date_label(last)
        ),
        _ => ", not on days yet".to_string(),
    };
    let seed = detail
        .plan
        .seed
        .map_or_else(String::new, |s| format!(" Seed {s}."));
    format!("{count}{span}.{seed}")
}

/// The latest active plan when `id` is `None`, otherwise that plan.
#[component]
fn PlanView(id: ReadSignal<Option<Uuid>>) -> Element {
    let mut plan = use_get_plan(id);
    let offset = use_local_offset();
    let today = use_today();
    let toast = use_toast();
    let navigator = use_navigator();
    // The day whose re-roll is in flight, so its button shows a spinner.
    let mut rerolling = use_signal(|| None as Option<Uuid>);
    // The day being swapped; `Some` mounts the recipe picker.
    let mut swapping = use_signal(|| None as Option<PlannedMealDetail>);
    let mut starring = use_signal(|| false);
    let mut opening_list = use_signal(|| false);
    // The dinner whose move is in flight, and whether the whole plan is being placed.
    let mut moving = use_signal(|| None as Option<Uuid>);
    let mut placing = use_signal(|| false);
    // The dinner being dragged to another day.
    let mut dragging = use_signal(|| None as Option<Uuid>);

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
                        format!(
                            "{} is now {}.",
                            dinner_day_label(meal.day_index, meal.scheduled_date),
                            meal.title
                        ),
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

    let mut move_meal = move |id: Uuid, date: Option<Date>| {
        if moving().is_some() || placing() {
            return;
        }
        moving.set(Some(id));
        spawn(async move {
            match schedule_planned_meal(id, date).await {
                Ok(meals) => {
                    plan.restart();
                    if let Some(meal) = meals.iter().find(|m| m.id == id) {
                        toast.show(
                            Some("Moved".to_string()),
                            format!(
                                "{} is on {}.",
                                meal.title,
                                dinner_day_label(meal.day_index, meal.scheduled_date)
                            ),
                            Tone::Green,
                        );
                    }
                }
                Err(e) => toast.show(
                    Some("Couldn't move it".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            moving.set(None);
        });
    };

    let mut put_on_days = move |plan_id: Uuid, start: Date| {
        if moving().is_some() || placing() {
            return;
        }
        placing.set(true);
        spawn(async move {
            match schedule_plan(plan_id, start).await {
                Ok(_) => {
                    plan.restart();
                    toast.show(
                        Some("Put on days".to_string()),
                        format!("Day 1 is on {}.", day_label(start)),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't put the dinners on days".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            placing.set(false);
        });
    };

    let on_swapped = move |updated: PlannedMealDetail| {
        swapping.set(None);
        plan.restart();
        toast.show(
            Some("Swapped".to_string()),
            format!(
                "{} is now {}.",
                dinner_day_label(updated.day_index, updated.scheduled_date),
                updated.title
            ),
            Tone::Green,
        );
    };

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
                    Heading { level: 1, "Planned Meals" }
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

            if let Some(d) = detail {
                ScheduleCard {
                    key: "{d.plan.id}",
                    start_date: d.plan.start_date,
                    unscheduled: d.meals.iter().filter(|m| m.scheduled_date.is_none()).count(),
                    busy: placing(),
                    on_schedule: {
                        let plan_id = d.plan.id;
                        move |start: Date| put_on_days(plan_id, start)
                    },
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
                        Text { muted: true, "Spin the roller and your dinners land here. Archived weeks are in History." }
                        div { class: "mr-empty__actions",
                            Link { to: Route::Roll {}, class: "mr-btn mr-btn--solid mr-btn--lg", "Roll a week" }
                            Link { to: Route::History {}, class: "mr-btn mr-btn--outline mr-btn--lg", "Open History" }
                        }
                    }
                },
                Some(Ok(Some(d))) => rsx! {
                    if d.meals.is_empty() {
                        Text { muted: true, "This plan has no days." }
                    } else {
                        div { class: "mr-plan-week",
                            for meal in d.meals.iter().cloned() {
                                DayCard {
                                    key: "{meal.id}",
                                    tonight: meal.scheduled_date.is_some() && today() == meal.scheduled_date,
                                    rerolling: rerolling() == Some(meal.id),
                                    locked: rerolling().is_some(),
                                    moving: moving().is_some() || placing(),
                                    drop_ready: dragging().is_some_and(|from| from != meal.id),
                                    on_reroll: {
                                        let id = meal.id;
                                        move |_| reroll(id)
                                    },
                                    on_swap: {
                                        let meal = meal.clone();
                                        move |_| swapping.set(Some(meal.clone()))
                                    },
                                    on_move: {
                                        let id = meal.id;
                                        move |date: Option<Date>| move_meal(id, date)
                                    },
                                    on_drag_start: {
                                        let id = meal.id;
                                        move |_| dragging.set(Some(id))
                                    },
                                    on_drag_end: move |_| dragging.set(None),
                                    on_drop: {
                                        let id = meal.id;
                                        let date = meal.scheduled_date;
                                        move |_| {
                                            let from = dragging();
                                            dragging.set(None);
                                            if let Some(from) = from {
                                                if from != id {
                                                    move_meal(from, date);
                                                }
                                            }
                                        }
                                    },
                                    meal,
                                }
                            }
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
