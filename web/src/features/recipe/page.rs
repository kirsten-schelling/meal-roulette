use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Alert, Button, ButtonVariant, CellAlign, Column, DataTable, FilterField, Heading, SearchBar,
    Spinner, Text, Tone,
};
use crate::features::recipe::hooks::use_recipe::use_get_recipes;
use dioxus::prelude::*;
use domain::recipe::Recipe;

#[component]
pub fn Recipes() -> Element {
    let recipes: Resource<Result<Vec<Recipe>, ServerFnError>> = use_get_recipes();

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
        Column::new("Time", |r: &Recipe| rsx! { "{r.time_mins} min" }).align(CellAlign::End),
    ];

    rsx! {
        div { class: "mr-page",
            // ── Header: titles on the left, the (not-yet-wired) add button right.
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "Recipes" }
                    Text { muted: true,
                        "Your recipe library — everything the roulette can pick from."
                    }
                }
                Button {
                    variant: ButtonVariant::Solid,
                    onclick: move |_| {
                        // TODO: open the add-recipe flow.
                    },
                    "＋ Add recipe"
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

            // ── The table, driven by the resource's three states.
            match recipes() {
                None => rsx! {
                    div { class: "mr-tablepanel mr-tablestate",
                        Spinner {}
                    }
                },
                Some(Err(e)) => rsx! {
                    Alert { tone: Tone::Red, title: "Couldn't load recipes",
                        "{e}"
                    }
                },
                Some(Ok(list)) => rsx! {
                    div { class: "mr-tablepanel",
                        DataTable::<Recipe> {
                            columns: columns.clone(),
                            rows: list,
                            empty: "No recipes yet — add your first one.".to_string(),
                        }
                    }
                },
            }
        }
    }
}
