use crate::components::ui::{
    Alert, Button, ButtonVariant, CellAlign, Column, ConfirmDialog, DataTable, Heading, Icon,
    IconButton, IconName, Spinner, Text, Tone,
};
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::ingredient::hooks::use_ingredient::{
    delete_ingredient, delete_ingredient_cascade, get_ingredient_references, use_get_ingredients,
};
use crate::features::library::components::{
    BlockedDeleteDialog, IngredientDialog, LookupCard, ManageListsDialog, ManageTab,
};
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::tag::hooks::use_tag::use_get_tags;
use crate::features::unit::hooks::use_unit::use_get_units;
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::ingredient::Ingredient;
use domain::usage::Reference;

/// Flattens a lookup fetch into the names the rail cards show — the error
/// becomes a plain message so the card's props stay simple.
fn names_from<T>(
    fetched: &Option<Result<Vec<T>, ServerFnError>>,
    name: impl Fn(&T) -> String,
) -> Option<Result<Vec<String>, String>> {
    match fetched {
        None => None,
        Some(Ok(items)) => Some(Ok(items.iter().map(name).collect())),
        Some(Err(e)) => Some(Err(error_message(e))),
    }
}

/// Which ingredient dialog is open: none, a fresh one, or one editing a row.
#[derive(Clone, PartialEq)]
enum IngredientForm {
    Closed,
    Adding,
    Editing(Ingredient),
}

/// The Library: everything recipes are built from. Ingredients are the main
/// table (they grow into the base ingredient library); the tiny lookup lists
/// (cuisines / proteins / units / tags) live in a rail, managed through a dialog.
#[component]
pub fn Library() -> Element {
    let mut ingredients = use_get_ingredients();
    let mut cuisines = use_get_cuisines();
    let mut proteins = use_get_proteins();
    let mut units = use_get_units();
    let mut tags = use_get_tags();
    let toast = use_toast();

    // Which manage tab is open, if any — `None` means the dialog is closed.
    let mut managing = use_signal(|| None as Option<ManageTab>);
    let mut form = use_signal(|| IngredientForm::Closed);
    let mut deleting = use_signal(|| None as Option<Ingredient>);
    let mut delete_busy = use_signal(|| false);
    let mut blocked = use_signal(|| None as Option<(Ingredient, Vec<Reference>)>);

    let mut delete_confirmed = move |ingredient: Ingredient| {
        if delete_busy() {
            return;
        }
        delete_busy.set(true);
        spawn(async move {
            match delete_ingredient(ingredient.id).await {
                Ok(_) => {
                    ingredients.restart();
                    toast.show(
                        Some("Deleted".to_string()),
                        format!("{} was deleted.", ingredient.name),
                        Tone::Green,
                    );
                }
                Err(e) => match get_ingredient_references(ingredient.id).await {
                    Ok(references) if !references.is_empty() => {
                        blocked.set(Some((ingredient, references)));
                    }
                    _ => toast.show(
                        Some("Couldn't delete".to_string()),
                        error_message(&e),
                        Tone::Red,
                    ),
                },
            }
            deleting.set(None);
            delete_busy.set(false);
        });
    };

    let mut cascade_confirmed = move |ingredient: Ingredient| {
        if delete_busy() {
            return;
        }
        delete_busy.set(true);
        spawn(async move {
            match delete_ingredient_cascade(ingredient.id).await {
                Ok(_) => {
                    ingredients.restart();
                    toast.show(
                        Some("Deleted".to_string()),
                        format!("{} was deleted.", ingredient.name),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't delete".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            blocked.set(None);
            delete_busy.set(false);
        });
    };

    let columns: Vec<Column<Ingredient>> = vec![
        Column::new("Ingredient", |i: &Ingredient| {
            rsx! {
                span { class: "mr-cell-strong", "{i.name}" }
            }
        }),
        Column::new("Purchase size", |i: &Ingredient| match i.purchase_size {
            Some(size) => rsx! { "{size} {i.unit}" },
            None => rsx! {
                span { class: "mr-text--muted", "Not set" }
            },
        })
        .align(CellAlign::End),
        Column::new("", move |i: &Ingredient| {
            let to_edit = i.clone();
            let to_delete = i.clone();
            rsx! {
                div { class: "mr-rowactions",
                    IconButton {
                        label: format!("Edit {}", i.name),
                        onclick: move |_| form.set(IngredientForm::Editing(to_edit.clone())),
                        Icon { name: IconName::Edit }
                    }
                    IconButton {
                        label: format!("Delete {}", i.name),
                        danger: true,
                        onclick: move |_| deleting.set(Some(to_delete.clone())),
                        Icon { name: IconName::X }
                    }
                }
            }
        })
        .align(CellAlign::End)
        .interactive(),
    ];

    let open_form = form();

    rsx! {
        div { class: "mr-page",
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "Library" }
                    Text { muted: true,
                        "Everything your recipes are built from: ingredients, cuisines, proteins, units and tags."
                    }
                }
                div { class: "mr-pagehead__actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| managing.set(Some(ManageTab::Cuisines)),
                        "Manage lists"
                    }
                    Button {
                        variant: ButtonVariant::Solid,
                        onclick: move |_| form.set(IngredientForm::Adding),
                        Icon { name: IconName::Plus }
                        "Add ingredient"
                    }
                }
            }

            if let Some(tab) = managing() {
                ManageListsDialog {
                    initial_tab: tab,
                    on_close: move |_| {
                        managing.set(None);
                        cuisines.restart();
                        proteins.restart();
                        units.restart();
                        tags.restart();
                        // A renamed unit shows in the ingredient table too.
                        ingredients.restart();
                    },
                }
            }

            match open_form {
                IngredientForm::Closed => rsx! {},
                IngredientForm::Adding => rsx! {
                    IngredientDialog {
                        on_close: move |_| form.set(IngredientForm::Closed),
                        on_saved: move |_| {
                            ingredients.restart();
                            units.restart();
                            form.set(IngredientForm::Closed);
                        },
                    }
                },
                IngredientForm::Editing(ingredient) => rsx! {
                    IngredientDialog {
                        existing: ingredient,
                        on_close: move |_| form.set(IngredientForm::Closed),
                        on_saved: move |_| {
                            ingredients.restart();
                            units.restart();
                            form.set(IngredientForm::Closed);
                            toast.show(
                                Some("Saved".to_string()),
                                "The ingredient is updated.".to_string(),
                                Tone::Green,
                            );
                        },
                    }
                },
            }

            if let Some(ingredient) = deleting() {
                ConfirmDialog {
                    title: format!("Delete {}?", ingredient.name),
                    confirm_label: "Delete".to_string(),
                    danger: true,
                    loading: delete_busy(),
                    on_confirm: move |_| delete_confirmed(ingredient.clone()),
                    on_cancel: move |_| {
                        if !delete_busy() {
                            deleting.set(None);
                        }
                    },
                    Text {
                        "This removes it for good. If a recipe or a shopping list still uses it, the delete is refused."
                    }
                }
            }

            if let Some((ingredient, references)) = blocked() {
                BlockedDeleteDialog {
                    name: ingredient.name.clone(),
                    references,
                    cascade_warning: Some(
                        "Delete anyway takes it out of these recipes and shopping lists, then deletes it. The recipes and lists themselves are kept."
                            .to_string(),
                    ),
                    busy: delete_busy(),
                    on_cascade: move |_| cascade_confirmed(ingredient.clone()),
                    on_close: move |_| {
                        if !delete_busy() {
                            blocked.set(None);
                        }
                    },
                }
            }

            div { class: "mr-library",
                div { class: "mr-library__main",
                    match ingredients() {
                        None => rsx! {
                            div { class: "mr-tablepanel mr-tablestate", Spinner {} }
                        },
                        Some(Err(e)) => rsx! {
                            Alert { tone: Tone::Red, title: "Couldn't load ingredients", "{error_message(&e)}" }
                        },
                        Some(Ok(list)) => rsx! {
                            div { class: "mr-tablepanel",
                                DataTable::<Ingredient> {
                                    columns: columns.clone(),
                                    rows: list,
                                    empty: "No ingredients yet. Add one, or let recipes create them as you go."
                                        .to_string(),
                                }
                            }
                        },
                    }
                }

                aside { class: "mr-library__rail",
                    LookupCard {
                        title: "Cuisines".to_string(),
                        names: names_from(&cuisines(), |c| c.cuisine.name.clone()),
                        on_manage: move |_| managing.set(Some(ManageTab::Cuisines)),
                    }
                    LookupCard {
                        title: "Proteins".to_string(),
                        names: names_from(&proteins(), |p| p.protein.name.clone()),
                        on_manage: move |_| managing.set(Some(ManageTab::Proteins)),
                    }
                    LookupCard {
                        title: "Units".to_string(),
                        names: names_from(&units(), |u| u.unit.name.clone()),
                        on_manage: move |_| managing.set(Some(ManageTab::Units)),
                    }
                    LookupCard {
                        title: "Tags".to_string(),
                        names: names_from(&tags(), |t| t.tag.name.clone()),
                        on_manage: move |_| managing.set(Some(ManageTab::Tags)),
                    }
                }
            }
        }
    }
}
