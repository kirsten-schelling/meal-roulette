//! One recipe: its details in a summary panel, and its ingredient lines in a table
//! that can be added to, edited, and trimmed in place.

use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Alert, Badge, Button, ButtonVariant, CellAlign, Column, ComboOption, ConfirmDialog, DataTable,
    Heading, Icon, IconButton, IconName, Spinner, Text, Tone,
};
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::library::components::{ManageListsDialog, ManageTab};
use crate::features::recipe::components::ingredient_composer::{parse_qty, IngredientComposer};
use crate::features::recipe::components::ingredient_line::RecipeLine;
use crate::features::recipe::components::recipe_fields::options_from;
use crate::features::recipe::components::{EditLineDialog, EditRecipeDialog};
use crate::features::recipe::hooks::use_recipe::{
    add_recipe_ingredient, delete_recipe, remove_recipe_ingredient, use_get_recipe_detail,
};
use crate::features::recipe::page::healthiness_badge;
use crate::features::unit::hooks::use_unit::use_get_units;
use crate::hooks::use_toast::use_toast;
use crate::models::{date_label, error_message};
use crate::router::Route;
use dioxus::prelude::*;
use domain::recipe::RecipeDetail;
use domain::recipe_ingredient::{RecipeIngredient, RecipeIngredientEntry};
use uuid::Uuid;

fn summary_item(label: &str, value: Element) -> Element {
    rsx! {
        div { class: "mr-summary__item",
            dt { class: "mr-summary__label", "{label}" }
            dd { class: "mr-summary__value", {value} }
        }
    }
}

fn summary(detail: &RecipeDetail) -> Element {
    let o = &detail.overview;
    let named = |value: &Option<String>| match value {
        Some(v) => rsx! { "{v}" },
        None => rsx! {
            span { class: "mr-text--muted", "None" }
        },
    };
    let tags = if o.tags.is_empty() {
        rsx! {
            span { class: "mr-text--muted", "No tags" }
        }
    } else {
        rsx! {
            div { class: "mr-tagcell",
                for tag in o.tags.iter() {
                    Badge { key: "{tag.id}", "{tag.name}" }
                }
            }
        }
    };
    rsx! {
        dl { class: "mr-summary",
            {summary_item("Difficulty", rsx! { DifficultyBadge { difficulty: o.recipe.difficulty } })}
            {summary_item("Healthiness", healthiness_badge(o.recipe.healthiness))}
            {summary_item("Time", rsx! { "{o.recipe.time_mins} min" })}
            {summary_item("Servings", rsx! { "{o.recipe.servings}" })}
            {summary_item("Cuisine", named(&o.cuisine))}
            {summary_item("Main protein", named(&o.protein))}
            div { class: "mr-summary__item mr-summary__item--wide",
                dt { class: "mr-summary__label", "Tags" }
                dd { class: "mr-summary__value", {tags} }
            }
        }
    }
}

#[component]
pub fn RecipePage(id: Uuid) -> Element {
    let mut detail = use_get_recipe_detail(id);
    let ingredients = use_get_ingredients();
    let mut units = use_get_units();
    let toast = use_toast();
    let navigator = use_navigator();

    let mut editing_details = use_signal(|| false);
    let mut editing_line = use_signal(|| None as Option<RecipeIngredient>);
    let mut removing_line = use_signal(|| None as Option<RecipeIngredient>);
    let mut composing = use_signal(|| false);
    let mut adding_busy = use_signal(|| false);
    let mut confirm_delete = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let mut managing_units = use_signal(|| false);

    let add_line = move |line: RecipeLine| {
        let Ok(qty) = parse_qty(&line.qty) else {
            return;
        };
        let entry = RecipeIngredientEntry {
            ingredient: line.ingredient,
            qty,
            note: (!line.note.is_empty()).then_some(line.note),
            is_optional: line.optional,
        };
        adding_busy.set(true);
        spawn(async move {
            match add_recipe_ingredient(id, entry).await {
                Ok(added) => {
                    detail.restart();
                    units.restart();
                    toast.show(
                        Some("Added".to_string()),
                        format!("{} is in the recipe.", added.ingredient_name),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't add the ingredient".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            adding_busy.set(false);
        });
    };

    let remove_line = move |line: RecipeIngredient| {
        spawn(async move {
            match remove_recipe_ingredient(line.recipe_id, line.ingredient_id).await {
                Ok(_) => {
                    detail.restart();
                    toast.show(
                        Some("Removed".to_string()),
                        format!("{} is no longer in the recipe.", line.ingredient_name),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't remove it".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            removing_line.set(None);
        });
    };

    let mut delete_this = move || {
        if deleting() {
            return;
        }
        deleting.set(true);
        spawn(async move {
            match delete_recipe(id).await {
                Ok(_) => {
                    navigator.push(Route::Recipes {});
                }
                Err(e) => {
                    toast.show(
                        Some("Couldn't delete".to_string()),
                        error_message(&e),
                        Tone::Red,
                    );
                    confirm_delete.set(false);
                }
            }
            deleting.set(false);
        });
    };

    let columns: Vec<Column<RecipeIngredient>> = vec![
        Column::new("Ingredient", |l: &RecipeIngredient| {
            rsx! {
                span { class: "mr-cell-strong", "{l.ingredient_name}" }
                if l.is_optional {
                    " "
                    Badge { "Optional" }
                }
            }
        }),
        Column::new("Amount", |l: &RecipeIngredient| {
            rsx! { "{l.qty.normalize()} {l.unit}" }
        })
        .align(CellAlign::End),
        Column::new("Note", |l: &RecipeIngredient| match &l.note {
            Some(note) => rsx! { "{note}" },
            None => rsx! {
                span { class: "mr-text--muted", "None" }
            },
        }),
        Column::new("", move |l: &RecipeIngredient| {
            let to_edit = l.clone();
            let to_remove = l.clone();
            rsx! {
                div { class: "mr-rowactions",
                    IconButton {
                        label: format!("Edit {}", l.ingredient_name),
                        onclick: move |_| editing_line.set(Some(to_edit.clone())),
                        Icon { name: IconName::Edit }
                    }
                    IconButton {
                        label: format!("Remove {}", l.ingredient_name),
                        danger: true,
                        onclick: move |_| removing_line.set(Some(to_remove.clone())),
                        Icon { name: IconName::X }
                    }
                }
            }
        })
        .align(CellAlign::End)
        .interactive(),
    ];

    let ingredient_options = options_from(&ingredients(), |i| {
        ComboOption::new(i.id, i.name.clone()).with_hint(i.unit.clone())
    });
    let unit_options = options_from(&units(), |u| {
        ComboOption::new(u.unit.id, u.unit.name.clone())
    });

    let loaded = match &*detail.read() {
        None => {
            return rsx! {
                div { class: "mr-page",
                    div { class: "mr-tablestate", Spinner {} }
                }
            };
        }
        Some(Err(e)) => {
            let message = error_message(e);
            return rsx! {
                div { class: "mr-page",
                    Link { to: Route::Recipes {}, class: "mr-backlink",
                        Icon { name: IconName::ArrowLeft }
                        "Recipes"
                    }
                    Alert { tone: Tone::Red, title: "Couldn't load this recipe", "{message}" }
                }
            };
        }
        Some(Ok(d)) => d.clone(),
    };
    let recipe = loaded.overview.recipe.clone();
    let line_count = loaded.ingredients.len();
    let taken: Vec<String> = loaded
        .ingredients
        .iter()
        .map(|l| l.ingredient_name.to_lowercase())
        .collect();
    let added_on = date_label(recipe.created_at.date());

    rsx! {
        div { class: "mr-page",
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Link { to: Route::Recipes {}, class: "mr-backlink",
                        Icon { name: IconName::ArrowLeft }
                        "Recipes"
                    }
                    Heading { level: 1, "{recipe.title}" }
                    Text { muted: true, "Added {added_on}" }
                }
                div { class: "mr-pagehead__actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| editing_details.set(true),
                        "Edit details"
                    }
                    Button {
                        variant: ButtonVariant::Danger,
                        onclick: move |_| confirm_delete.set(true),
                        "Delete"
                    }
                }
            }

            div { class: "mr-recipe-page",
                section { class: "mr-card mr-recipe-page__summary",
                    div { class: "mr-card__header", "Summary" }
                    div { class: "mr-card__body", {summary(&loaded)} }
                }

                section { class: "mr-card mr-recipe-page__ingredients",
                    div { class: "mr-card__header", "Ingredients ({line_count})" }
                    div { class: "mr-card__body",
                        DataTable::<RecipeIngredient> {
                            columns: columns.clone(),
                            rows: loaded.ingredients.clone(),
                            empty: "No ingredients yet. Add the first one below.".to_string(),
                        }
                        if composing() {
                            div { class: "mr-recipe-page__composer",
                                IngredientComposer {
                                    ingredient_options: ingredient_options.clone(),
                                    unit_options: unit_options.clone(),
                                    taken: taken.clone(),
                                    busy: adding_busy(),
                                    on_manage_units: move |_| managing_units.set(true),
                                    on_add: add_line,
                                }
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: move |_| composing.set(false),
                                    "Done adding"
                                }
                            }
                        } else {
                            button {
                                r#type: "button",
                                class: "mr-add-link",
                                onclick: move |_| composing.set(true),
                                Icon { name: IconName::Plus }
                                "Add ingredient"
                            }
                        }
                    }
                }
            }
        }

        if editing_details() {
            EditRecipeDialog {
                overview: loaded.overview.clone(),
                on_close: move |_| editing_details.set(false),
                on_saved: move |_| {
                    editing_details.set(false);
                    detail.restart();
                    toast.show(
                        Some("Saved".to_string()),
                        "The recipe's details are updated.".to_string(),
                        Tone::Green,
                    );
                },
            }
        }

        if let Some(line) = editing_line() {
            EditLineDialog {
                line,
                on_close: move |_| editing_line.set(None),
                on_saved: move |_| {
                    editing_line.set(None);
                    detail.restart();
                },
            }
        }

        if let Some(line) = removing_line() {
            ConfirmDialog {
                title: format!("Remove {}?", line.ingredient_name),
                confirm_label: "Remove".to_string(),
                danger: true,
                on_confirm: move |_| remove_line(line.clone()),
                on_cancel: move |_| removing_line.set(None),
                Text { "It comes off this recipe only; the ingredient stays in your library." }
            }
        }

        if confirm_delete() {
            ConfirmDialog {
                title: format!("Delete “{}”?", recipe.title),
                confirm_label: "Delete".to_string(),
                danger: true,
                loading: deleting(),
                on_confirm: move |_| delete_this(),
                on_cancel: move |_| confirm_delete.set(false),
                Text {
                    "This removes it for good, along with its ingredient lines and tags. A recipe that is on a meal plan or in the cooking log can't be deleted."
                }
            }
        }

        if managing_units() {
            ManageListsDialog {
                initial_tab: ManageTab::Units,
                on_close: move |_| {
                    managing_units.set(false);
                    units.restart();
                },
            }
        }
    }
}
