use domain::recipe::RecipeOverview;
use std::ops::RangeInclusive;

/// The filter types the Recipes search bar offers, by the label its chips carry.
pub const TAG: &str = "Tag";
pub const CUISINE: &str = "Cuisine";
pub const PROTEIN: &str = "Protein";
pub const DIFFICULTY: &str = "Difficulty";
pub const MAX_TIME: &str = "Max time";
pub const SERVINGS: &str = "Servings";
pub const HEALTHINESS: &str = "Healthiness";

fn contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// "3" or "2-4" (also with an en dash): the values a number filter accepts.
fn number_range(value: &str) -> Option<RangeInclusive<i16>> {
    let value = value.trim();
    match value.split_once(['-', '\u{2013}']) {
        Some((low, high)) => Some(low.trim().parse().ok()?..=high.trim().parse().ok()?),
        None => {
            let n = value.parse().ok()?;
            Some(n..=n)
        }
    }
}

fn matches_one(recipe: &RecipeOverview, field: &str, value: &str) -> bool {
    let r = &recipe.recipe;
    match field {
        TAG => recipe.tags.iter().any(|t| contains(&t.name, value)),
        CUISINE => recipe
            .cuisine
            .as_deref()
            .is_some_and(|c| contains(c, value)),
        PROTEIN => recipe
            .protein
            .as_deref()
            .is_some_and(|p| contains(p, value)),
        DIFFICULTY => number_range(value).is_some_and(|range| range.contains(&r.difficulty)),
        MAX_TIME => value
            .trim()
            .parse::<i16>()
            .is_ok_and(|max| r.time_mins <= max),
        SERVINGS => number_range(value).is_some_and(|range| range.contains(&r.servings)),
        HEALTHINESS => number_range(value)
            .is_some_and(|range| r.healthiness.is_some_and(|h| range.contains(&h))),
        _ => true,
    }
}

fn matches_query(recipe: &RecipeOverview, query: &str) -> bool {
    let query = query.trim();
    query.is_empty()
        || contains(&recipe.recipe.title, query)
        || recipe
            .cuisine
            .as_deref()
            .is_some_and(|c| contains(c, query))
        || recipe
            .protein
            .as_deref()
            .is_some_and(|p| contains(p, query))
        || recipe.tags.iter().any(|t| contains(&t.name, query))
}

/// The recipes matching a free-text search plus the search bar's filter chips
/// ("Cuisine: thai"). Two chips of the same kind widen the match ("Thai or Italian"),
/// except tags, which narrow it: a recipe can carry several tags, so "Quick" and
/// "Vegetarian" means both.
pub fn matching(recipes: &[RecipeOverview], query: &str, chips: &[String]) -> Vec<RecipeOverview> {
    let filters: Vec<(&str, &str)> = chips
        .iter()
        .filter_map(|chip| chip.split_once(": "))
        .collect();
    recipes
        .iter()
        .filter(|recipe| matches_query(recipe, query))
        .filter(|recipe| {
            let mut fields: Vec<&str> = filters.iter().map(|(field, _)| *field).collect();
            fields.sort_unstable();
            fields.dedup();
            fields.iter().all(|field| {
                let mut values = filters
                    .iter()
                    .filter(|(f, _)| f == field)
                    .map(|(_, value)| value);
                if *field == TAG {
                    values.all(|value| matches_one(recipe, field, value))
                } else {
                    values.any(|value| matches_one(recipe, field, value))
                }
            })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::recipe::Recipe;
    use domain::tag::TagRef;
    use uuid::Uuid;

    fn recipe(
        title: &str,
        cuisine: &str,
        difficulty: i16,
        time: i16,
        tags: &[&str],
    ) -> RecipeOverview {
        RecipeOverview {
            recipe: Recipe {
                id: Uuid::nil(),
                household_id: None,
                title: title.to_string(),
                difficulty,
                time_mins: time,
                servings: 2,
                healthiness: Some(4),
                main_protein_id: None,
                cuisine_id: None,
                created_at: time::OffsetDateTime::UNIX_EPOCH,
                updated_at: time::OffsetDateTime::UNIX_EPOCH,
            },
            cuisine: Some(cuisine.to_string()),
            protein: Some("Chicken".to_string()),
            tags: tags
                .iter()
                .map(|name| TagRef {
                    id: Uuid::nil(),
                    name: name.to_string(),
                })
                .collect(),
        }
    }

    fn titles(found: Vec<RecipeOverview>) -> Vec<String> {
        found.into_iter().map(|r| r.recipe.title).collect()
    }

    fn pool() -> Vec<RecipeOverview> {
        vec![
            recipe("Pad Thai", "Thai", 2, 25, &["Quick"]),
            recipe("Lasagne", "Italian", 4, 90, &["Vegetarian"]),
            recipe("Green curry", "Thai", 3, 40, &["Quick", "Vegetarian"]),
        ]
    }

    fn chips(list: &[&str]) -> Vec<String> {
        list.iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn free_text_searches_title_and_lookups() {
        assert_eq!(titles(matching(&pool(), "curry", &[])), ["Green curry"]);
        assert_eq!(titles(matching(&pool(), "italian", &[])), ["Lasagne"]);
    }

    #[test]
    fn same_kind_chips_widen_but_tags_narrow() {
        let either = matching(&pool(), "", &chips(&["Cuisine: Thai", "Cuisine: Italian"]));
        assert_eq!(either.len(), 3);
        let both = matching(&pool(), "", &chips(&["Tag: quick", "Tag: vegetarian"]));
        assert_eq!(titles(both), ["Green curry"]);
    }

    #[test]
    fn number_filters_take_values_ranges_and_maximums() {
        assert_eq!(
            titles(matching(&pool(), "", &chips(&["Difficulty: 4"]))),
            ["Lasagne"]
        );
        assert_eq!(matching(&pool(), "", &chips(&["Difficulty: 2-3"])).len(), 2);
        assert_eq!(matching(&pool(), "", &chips(&["Max time: 40"])).len(), 2);
        assert!(matching(&pool(), "", &chips(&["Difficulty: easy"])).is_empty());
    }
}
