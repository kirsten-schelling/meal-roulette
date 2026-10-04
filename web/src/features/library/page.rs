use crate::components::ui::{
    Alert, Button, ButtonVariant, CellAlign, Column, DataTable, Heading, IconButton, Spinner, Text,
    Tone,
};
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::library::components::{
    AddIngredientDialog, LookupCard, ManageListsDialog, ManageTab,
};
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::unit::hooks::use_unit::use_get_units;
use dioxus::prelude::*;
use domain::ingredient::Ingredient;

/// Flattens a lookup fetch into the names the rail cards show — the error
/// becomes a plain message so the card's props stay simple.
fn names_from<T>(
    fetched: &Option<Result<Vec<T>, ServerFnError>>,
    name: impl Fn(&T) -> String,
) -> Option<Result<Vec<String>, String>> {
    match fetched {
        None => None,
        Some(Ok(items)) => Some(Ok(items.iter().map(name).collect())),
        Some(Err(e)) => Some(Err(e.to_string())),
    }
}

/// The Library: everything recipes are built from. Ingredients are the main
/// table (they grow into the base ingredient library); the tiny lookup lists
/// (cuisines / proteins / units) live in a rail, managed through a dialog.
#[component]
pub fn Library() -> Element {
    let mut ingredients = use_get_ingredients();
    let mut cuisines = use_get_cuisines();
    let mut proteins = use_get_proteins();
    let mut units = use_get_units();

    // Which manage tab is open, if any — `None` means the dialog is closed.
    let mut managing = use_signal(|| None as Option<ManageTab>);
    let mut adding = use_signal(|| false);

    let columns: Vec<Column<Ingredient>> = vec![
        Column::new("Ingredient", |i: &Ingredient| {
            rsx! {
                span { class: "mr-cell-strong", "{i.name}" }
            }
        }),
        Column::new("Purchase size", |i: &Ingredient| {
            rsx! { "{i.purchase_size} {i.unit}" }
        })
        .align(CellAlign::End),
        // Per-row actions — display-only until ingredient editing is wired up.
        Column::new("", |_i: &Ingredient| {
            rsx! {
                div { class: "mr-rowactions",
                    IconButton { label: "Edit ingredient".to_string(), onclick: move |_| {}, "✎" }
                    IconButton {
                        label: "Delete ingredient".to_string(),
                        danger: true,
                        onclick: move |_| {},
                        "\u{00d7}"
                    }
                }
            }
        })
        .align(CellAlign::End),
    ];

    rsx! {
        div { class: "mr-page",
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "Library" }
                    Text { muted: true,
                        "Everything your recipes are built from: ingredients, cuisines, proteins and units."
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
                        onclick: move |_| adding.set(true),
                        "＋ Add ingredient"
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
                    },
                }
            }

            if adding() {
                AddIngredientDialog {
                    on_close: move |_| adding.set(false),
                    on_saved: move |_| {
                        ingredients.restart();
                        adding.set(false);
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
                            Alert { tone: Tone::Red, title: "Couldn't load ingredients", "{e}" }
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
                }
            }
        }
    }
}
