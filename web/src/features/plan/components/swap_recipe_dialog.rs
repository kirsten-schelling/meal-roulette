//! Pick a specific recipe for one day. Lists the library with quick filters
//! (title, cuisine, protein, difficulty, time) and hides recipes already on
//! the plan, since a week never repeats a dinner.

use std::collections::HashMap;

use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Button, ButtonVariant, Input, Modal, Select, SelectOption, Size, Spinner, Text, Tone,
};
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::plan::hooks::use_plan::swap_planned_meal;
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::recipe::hooks::use_recipe::use_get_recipes;
use crate::hooks::use_toast::use_toast;
use crate::models::{day_label, difficulty_label, error_message};
use dioxus::prelude::*;
use domain::planned_meal::PlannedMealDetail;
use domain::recipe::Recipe;
use uuid::Uuid;

/// Lookup names keyed by id. Empty while the list is loading or failed.
fn lookup_names<T>(
    fetched: &Option<Result<Vec<T>, ServerFnError>>,
    pair: impl Fn(&T) -> (Uuid, String),
) -> HashMap<Uuid, String> {
    match fetched {
        Some(Ok(list)) => list.iter().map(pair).collect(),
        _ => HashMap::new(),
    }
}

/// An "any" entry followed by the lookup values in name order.
fn filter_options(any_label: &str, names: &HashMap<Uuid, String>) -> Vec<SelectOption> {
    let mut sorted: Vec<(&Uuid, &String)> = names.iter().collect();
    sorted.sort_by(|a, b| a.1.cmp(b.1));
    std::iter::once(SelectOption::new("", any_label))
        .chain(
            sorted
                .into_iter()
                .map(|(id, name)| SelectOption::new(id.to_string(), name.clone())),
        )
        .collect()
}

fn difficulty_options() -> Vec<SelectOption> {
    std::iter::once(SelectOption::new("", "Any difficulty"))
        .chain((1..=5).map(|d: i32| SelectOption::new(d.to_string(), difficulty_label(d))))
        .collect()
}

/// "Thai · Chicken · 25 min · 4 servings", skipping whatever the recipe lacks.
fn recipe_meta(
    recipe: &Recipe,
    cuisines: &HashMap<Uuid, String>,
    proteins: &HashMap<Uuid, String>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(name) = recipe.cuisine_id.and_then(|id| cuisines.get(&id)) {
        parts.push(name.clone());
    }
    if let Some(name) = recipe.main_protein_id.and_then(|id| proteins.get(&id)) {
        parts.push(name.clone());
    }
    parts.push(format!("{} min", recipe.time_mins));
    parts.push(format!("{} servings", recipe.servings));
    parts.join(" · ")
}

/// Mount it only while open, like `Modal`. `taken` holds the recipe ids already
/// on the plan; `on_swapped` fires with the updated day once the swap is saved.
#[component]
pub fn SwapRecipeDialog(
    meal: PlannedMealDetail,
    taken: Vec<Uuid>,
    on_close: EventHandler<()>,
    on_swapped: EventHandler<PlannedMealDetail>,
) -> Element {
    let recipes = use_get_recipes();
    let cuisines = use_get_cuisines();
    let proteins = use_get_proteins();
    let toast = use_toast();

    let mut query = use_signal(String::new);
    let mut cuisine = use_signal(String::new);
    let mut protein = use_signal(String::new);
    let mut difficulty = use_signal(String::new);
    let mut max_time = use_signal(String::new);
    // The recipe whose swap is in flight; the other Choose buttons wait.
    let mut saving = use_signal(|| None as Option<Uuid>);

    let cuisine_names = lookup_names(&cuisines(), |c| (c.cuisine.id, c.cuisine.name.clone()));
    let protein_names = lookup_names(&proteins(), |p| (p.protein.id, p.protein.name.clone()));

    let wanted_cuisine = Uuid::parse_str(&cuisine()).ok();
    let wanted_protein = Uuid::parse_str(&protein()).ok();
    let wanted_difficulty = difficulty().parse::<i16>().ok();
    let time_limit = max_time().trim().parse::<i16>().ok();
    let needle = query().trim().to_lowercase();

    let loading = recipes().is_none();
    let matches: Vec<Recipe> = match recipes() {
        Some(Ok(list)) => list
            .into_iter()
            .filter(|r| {
                !taken.contains(&r.id)
                    && (needle.is_empty() || r.title.to_lowercase().contains(&needle))
                    && wanted_cuisine.is_none_or(|id| r.cuisine_id == Some(id))
                    && wanted_protein.is_none_or(|id| r.main_protein_id == Some(id))
                    && wanted_difficulty.is_none_or(|d| r.difficulty == d)
                    && time_limit.is_none_or(|t| r.time_mins <= t)
            })
            .collect(),
        _ => Vec::new(),
    };

    let meal_id = meal.id;
    let mut choose = move |recipe_id: Uuid| {
        if saving().is_some() {
            return;
        }
        saving.set(Some(recipe_id));
        spawn(async move {
            match swap_planned_meal(meal_id, recipe_id).await {
                Ok(updated) => on_swapped.call(updated),
                Err(e) => {
                    toast.show(
                        Some("Couldn't swap".to_string()),
                        error_message(&e),
                        Tone::Red,
                    );
                    saving.set(None);
                }
            }
        });
    };

    rsx! {
        Modal {
            wide: true,
            overflow_visible: true,
            title: format!("Swap {}", day_label(meal.scheduled_date)),
            on_close: move |_| on_close.call(()),

            div { class: "mr-recipe-dialog__content",
                Text { small: true, muted: true,
                    "Currently {meal.title}. Pick any recipe that isn't already on the plan."
                }
                Input {
                    label: "Search".to_string(),
                    value: query(),
                    placeholder: "Recipe title…".to_string(),
                    oninput: move |e: FormEvent| query.set(e.value()),
                }
                div { class: "mr-form-grid",
                    Select {
                        label: "Cuisine".to_string(),
                        value: cuisine(),
                        options: filter_options("Any cuisine", &cuisine_names),
                        onchange: move |v| cuisine.set(v),
                    }
                    Select {
                        label: "Protein".to_string(),
                        value: protein(),
                        options: filter_options("Any protein", &protein_names),
                        onchange: move |v| protein.set(v),
                    }
                    Select {
                        label: "Difficulty".to_string(),
                        value: difficulty(),
                        options: difficulty_options(),
                        onchange: move |v| difficulty.set(v),
                    }
                    Input {
                        label: "Max time (mins)".to_string(),
                        kind: "number".to_string(),
                        value: max_time(),
                        placeholder: "Any".to_string(),
                        oninput: move |e: FormEvent| max_time.set(e.value()),
                    }
                }

                if loading {
                    div { class: "mr-tablestate", Spinner {} }
                } else if matches.is_empty() {
                    p { class: "mr-empty-note",
                        "No recipes match. Clear a filter, or add a recipe to the library."
                    }
                } else {
                    div { class: "mr-picklist",
                        for r in matches.iter() {
                            {
                                let rid = r.id;
                                let meta = recipe_meta(r, &cuisine_names, &protein_names);
                                let busy = saving() == Some(rid);
                                rsx! {
                                    div { key: "{r.id}", class: "mr-picklist__row",
                                        div { class: "mr-picklist__text",
                                            span { class: "mr-cell-strong", "{r.title}" }
                                            span { class: "mr-picklist__meta", "{meta}" }
                                        }
                                        DifficultyBadge { difficulty: r.difficulty }
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            size: Size::Sm,
                                            loading: busy,
                                            disabled: saving().is_some(),
                                            onclick: move |_| choose(rid),
                                            "Choose"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "mr-dialog-foot",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
            }
        }
    }
}
