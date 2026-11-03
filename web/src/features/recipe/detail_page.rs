//! One recipe: its details in a summary panel, and its ingredient lines in a table that
//! can be added to. The title, every detail and every line are edited in place by
//! clicking them.

use crate::components::ui::{
    Alert, Button, ButtonVariant, ComboOption, ConfirmDialog, Icon, IconName, Spinner, Text, Tone,
};
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::library::components::{ManageListsDialog, ManageTab};
use crate::features::recipe::components::ingredient_composer::{parse_qty, IngredientComposer};
use crate::features::recipe::components::ingredient_line::RecipeLine;
use crate::features::recipe::components::recipe_details::use_details_editor;
use crate::features::recipe::components::recipe_fields::{options_from, use_recipe_catalogs};
use crate::features::recipe::components::{CookMode, CopyRecipeDialog, IngredientTable};
use crate::features::recipe::hooks::use_recipe::{
    add_recipe_ingredient, delete_recipe, use_get_recipe_detail,
};
use crate::features::unit::hooks::use_unit::use_get_units;
use crate::hooks::use_toast::use_toast;
use crate::models::{date_label, error_message};
use crate::router::Route;
use dioxus::prelude::*;
use domain::recipe_ingredient::RecipeIngredientEntry;
use uuid::Uuid;

#[component]
pub fn RecipePage(id: Uuid) -> Element {
    let mut detail = use_get_recipe_detail(id);
    let ingredients = use_get_ingredients();
    let mut units = use_get_units();
    let catalogs = use_recipe_catalogs();
    let mut managing = use_signal(|| None as Option<ManageTab>);
    let editor = use_details_editor(detail, catalogs, managing);
    let toast = use_toast();
    let navigator = use_navigator();

    let mut composing = use_signal(|| false);
    let mut adding_busy = use_signal(|| false);
    let mut copying = use_signal(|| false);
    let mut cooking = use_signal(|| false);
    let mut confirm_delete = use_signal(|| false);
    let mut deleting = use_signal(|| false);

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
                    {editor.title(&loaded.overview)}
                    Text { muted: true, "Added {added_on} · Click anything to change it" }
                }
                div { class: "mr-pagehead__actions",
                    Button { onclick: move |_| cooking.set(true),
                        Icon { name: IconName::ChefHat }
                        "Cook"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| copying.set(true),
                        "Make a copy"
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
                    div { class: "mr-card__body", {editor.summary(&loaded.overview)} }
                }

                section { class: "mr-card mr-recipe-page__ingredients",
                    div { class: "mr-card__header", "Ingredients ({line_count})" }
                    div { class: "mr-card__body",
                        IngredientTable {
                            lines: loaded.ingredients.clone(),
                            on_changed: move |_| detail.restart(),
                        }
                        if composing() {
                            div { class: "mr-recipe-page__composer",
                                IngredientComposer {
                                    ingredient_options: ingredient_options.clone(),
                                    unit_options: unit_options.clone(),
                                    taken: taken.clone(),
                                    busy: adding_busy(),
                                    on_manage_units: move |_| managing.set(Some(ManageTab::Units)),
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

        if cooking() {
            CookMode {
                title: recipe.title.clone(),
                ingredients: loaded.ingredients.clone(),
                on_exit: move |_| cooking.set(false),
            }
        }

        if copying() {
            CopyRecipeDialog {
                recipe: recipe.clone(),
                on_close: move |_| copying.set(false),
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

        if let Some(tab) = managing() {
            ManageListsDialog {
                initial_tab: tab,
                on_close: move |_| {
                    managing.set(None);
                    units.restart();
                    catalogs.restart();
                },
            }
        }
    }
}
