use crate::components::ui::*;
use crate::features::recipe::hooks::use_recipe::use_get_recipe_overviews;
use crate::models::{difficulty_label, error_message};
use dioxus::prelude::*;
use domain::recipe::RecipeOverview;
use domain::roll::PreviewMeal;
use uuid::Uuid;

fn as_meal(overview: &RecipeOverview) -> PreviewMeal {
    PreviewMeal {
        recipe_id: overview.recipe.id,
        title: overview.recipe.title.clone(),
        difficulty: overview.recipe.difficulty,
        time_mins: overview.recipe.time_mins,
        cuisine: overview.cuisine.clone(),
        protein: overview.protein.clone(),
    }
}

/// Every recipe not already in the week, to pick one by hand for a day.
/// Mount it only while open so each open starts with an empty search.
#[component]
pub fn RecipePickDialog(
    day: usize,
    taken: Vec<Uuid>,
    on_pick: EventHandler<PreviewMeal>,
    on_close: EventHandler<()>,
) -> Element {
    let overviews = use_get_recipe_overviews();
    let mut query = use_signal(String::new);
    let needle = query().trim().to_lowercase();

    rsx! {
        Modal { on_close, title: format!("Pick a recipe for day {}", day + 1),
            Input {
                label: "Search".to_string(),
                value: query(),
                placeholder: "Recipe name".to_string(),
                oninput: move |e: FormEvent| query.set(e.value()),
            }
            match overviews() {
                None => rsx! {
                    Spinner {}
                },
                Some(Err(e)) => rsx! {
                    Alert { tone: Tone::Red, title: "Couldn't load recipes".to_string(), "{error_message(&e)}" }
                },
                Some(Ok(list)) => {
                    let mut available: Vec<&RecipeOverview> = list
                        .iter()
                        .filter(|o| !taken.contains(&o.recipe.id))
                        .filter(|o| o.recipe.title.to_lowercase().contains(&needle))
                        .collect();
                    available.sort_by_key(|o| o.recipe.title.to_lowercase());
                    rsx! {
                        if available.is_empty() {
                            Text { muted: true, "No other recipes match." }
                        } else {
                            ul { class: "mr-pick__list", aria_label: "Recipes",
                                for overview in available {
                                    li { key: "{overview.recipe.id}",
                                        button {
                                            r#type: "button",
                                            class: "mr-pick__row",
                                            onclick: {
                                                let meal = as_meal(overview);
                                                move |_| on_pick.call(meal.clone())
                                            },
                                            span { class: "mr-result__title", "{overview.recipe.title}" }
                                            span { class: "mr-result__meta",
                                                "{difficulty_label(i32::from(overview.recipe.difficulty))} · {overview.recipe.time_mins} min"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
