use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Alert, Badge, Button, ButtonVariant, CellAlign, Column, DataTable, FilterField, Heading,
    IconButton, SearchBar, Size, Spinner, Text, Tone,
};
use crate::features::recipe::components::AddRecipeDialog;
use crate::features::recipe::hooks::use_recipe::use_get_recipes;
use dioxus::prelude::*;
use domain::recipe::Recipe;

#[component]
pub fn Recipes() -> Element {
    let mut recipes: Resource<Result<Vec<Recipe>, ServerFnError>> = use_get_recipes();
    let mut adding = use_signal(|| false);
    // Row indexes ticked in the table. Display-only for now — the bulk Delete
    // button isn't wired to anything yet.
    let mut selected_rows = use_signal(Vec::<usize>::new);

    // The table's shape: each column says how to render its own cell from a
    // `&Recipe`. Nothing recipe-specific lives inside `DataTable` itself.
    let columns: Vec<Column<Recipe>> = vec![
        Column::new("Title", |r: &Recipe| {
            rsx! {
                span { class: "mr-cell-strong", "{r.title}" }
            }
        }),
        Column::new("Difficulty", |r: &Recipe| {
            rsx! {
                DifficultyBadge { difficulty: r.difficulty }
            }
        }),
        Column::new("Healthiness", |r: &Recipe| match r.healthiness {
            Some(h) => {
                let tone = match h {
                    1 | 2 => Tone::Red,
                    3 => Tone::Amber,
                    _ => Tone::Green,
                };
                rsx! {
                    Badge { tone, "{h}/5" }
                }
            }
            None => rsx! {
                span { class: "mr-text--muted", "—" }
            },
        }),
        Column::new("Time", |r: &Recipe| rsx! { "{r.time_mins} min" }).align(CellAlign::End),
        Column::new("Servings", |r: &Recipe| rsx! { "{r.servings}" }).align(CellAlign::End),
        // Per-row actions — display-only until editing/deleting is wired up.
        Column::new("", |_r: &Recipe| {
            rsx! {
                div { class: "mr-rowactions",
                    IconButton { label: "Edit recipe".to_string(), onclick: move |_| {}, "✎" }
                    IconButton {
                        label: "Delete recipe".to_string(),
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
            // ── Header: titles on the left, the add button on the right.
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "Recipes" }
                    Text { muted: true, "Your recipe library: everything the roulette can pick from." }
                }
                Button {
                    variant: ButtonVariant::Solid,
                    onclick: move |_| adding.set(true),
                    "＋ Add recipe"
                }
            }

            if adding() {
                AddRecipeDialog {
                    on_close: move |_| adding.set(false),
                    on_saved: move |_| {
                        recipes.restart();
                        adding.set(false);
                    },
                }
            }

            // ── Combined search + filter bar. Free text plus filter chips added
            //    from the "+" menu. The generic SearchBar owns the interaction;
            //    the page just supplies the recipe-specific filter types.
            div { class: "mr-page__filters",
                SearchBar {
                    placeholder: "Search recipes…",
                    filters: vec![
                        FilterField::new("Tag", "e.g. Vegetarian"),
                        FilterField::new("Cuisine", "e.g. Thai"),
                        FilterField::new("Protein", "e.g. Chicken"),
                        FilterField::new("Difficulty", "1–5"),
                        FilterField::new("Max time", "minutes"),
                        FilterField::new("Servings", "e.g. 4"),
                        FilterField::new("Healthiness", "1–5"),
                    ],
                }
            }

            if !selected_rows().is_empty() {
                div { class: "mr-bulkbar",
                    Text { small: true, "{selected_rows().len()} selected" }
                    Button { variant: ButtonVariant::Danger, size: Size::Sm, "Delete selected" }
                }
            }

            // ── The table, driven by the resource's three states.
            match recipes() {
                None => rsx! {
                    div { class: "mr-tablepanel mr-tablestate", Spinner {} }
                },
                Some(Err(e)) => rsx! {
                    Alert { tone: Tone::Red, title: "Couldn't load recipes", "{e}" }
                },
                Some(Ok(list)) => rsx! {
                    div { class: "mr-tablepanel",
                        DataTable::<Recipe> {
                            columns: columns.clone(),
                            rows: list,
                            empty: "No recipes yet. Add your first one.".to_string(),
                            selectable: true,
                            on_select: move |v: Vec<usize>| selected_rows.set(v),
                        }
                    }
                },
            }
        }
    }
}
