use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::Badge;
use crate::features::recipe::page::healthiness_badge;
use crate::router::Route;
use dioxus::prelude::*;
use domain::recipe::RecipeOverview;

/// Tags a card shows before summarising the rest as "+N".
const TAGS_SHOWN: usize = 3;

/// A stable hue (0-359) for a cuisine, so every recipe of one cuisine gets the
/// same placeholder colour. Not `DefaultHasher`: its output may differ between
/// the server and the browser build, which would break hydration.
fn cuisine_hue(cuisine: &str) -> u32 {
    cuisine
        .trim()
        .to_lowercase()
        .bytes()
        .fold(7u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32))
        % 360
}

fn initial(title: &str) -> String {
    title
        .trim()
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default()
}

/// One recipe as a card: a generated cuisine-coloured placeholder, the title in
/// display type, time and difficulty, healthiness and up to three tags. The
/// whole card opens the recipe.
#[component]
pub fn RecipeCard(overview: RecipeOverview) -> Element {
    let recipe = &overview.recipe;
    let extra = overview.tags.len().saturating_sub(TAGS_SHOWN);
    let (art_class, art_style) = match overview.cuisine.as_deref() {
        Some(cuisine) => (
            "mr-recipecard__art",
            format!("--mr-card-hue:{}", cuisine_hue(cuisine)),
        ),
        None => (
            "mr-recipecard__art mr-recipecard__art--plain",
            String::new(),
        ),
    };

    rsx! {
        article { class: "mr-recipecard",
            div { class: "{art_class}", style: "{art_style}", aria_hidden: "true",
                span { class: "mr-recipecard__initial", "{initial(&recipe.title)}" }
                if let Some(cuisine) = overview.cuisine.as_deref() {
                    span { class: "mr-recipecard__cuisine", "{cuisine}" }
                }
            }
            div { class: "mr-recipecard__body",
                h3 { class: "mr-recipecard__title",
                    Link {
                        class: "mr-recipecard__link",
                        to: Route::RecipePage { id: recipe.id },
                        "{recipe.title}"
                    }
                }
                div { class: "mr-recipecard__meta",
                    span { class: "mr-recipecard__time", "{recipe.time_mins} min" }
                    DifficultyBadge { difficulty: recipe.difficulty }
                    {healthiness_badge(recipe.healthiness)}
                }
                if !overview.tags.is_empty() {
                    div { class: "mr-tagcell",
                        for tag in overview.tags.iter().take(TAGS_SHOWN) {
                            Badge { key: "{tag.id}", "{tag.name}" }
                        }
                        if extra > 0 {
                            span { class: "mr-text--muted", "+{extra}" }
                        }
                    }
                }
            }
        }
    }
}
