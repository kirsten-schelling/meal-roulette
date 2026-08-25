use crate::components::difficulty_badge::DifficultyBadge;
use dioxus::prelude::*;
use domain::recipe::Recipe;

#[component]
pub fn RecipeTable(recipes: Vec<Recipe>) -> Element {
    rsx! {
        article {
            table {
                thead {
                    tr {
                        th { "Title" }
                        th { "Difficulty" }
                        th { "Time" }
                    }
                }
                tbody {
                    for r in recipes.iter() {
                        tr { key: "{r.id}",
                            td { "{r.title}" }
                            td {
                                DifficultyBadge { difficulty: r.difficulty }
                            }
                            td { "{r.time_mins} min" }
                        }
                    }
                }
            }
        }
    }
}
