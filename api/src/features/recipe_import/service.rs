use super::fetch::fetch_html;
use super::jsonld::{parse_recipe, ParsedRecipe};
use super::line::{canonical_unit, match_key, parse_line};
use crate::error::AppError;
use crate::features::{ingredient, unit};
use domain::ingredient::{Ingredient, IngredientEntry};
use domain::recipe_import::{ImportedIngredient, RecipeImport};
use domain::unit::{Unit, UnitEntry};
use sqlx::PgPool;
use std::collections::HashMap;

/// Unit given to a new ingredient whose line named none ("2 eggs").
const COUNT_UNIT: &str = "each";

pub async fn import_from_url(pool: &PgPool, raw_url: &str) -> Result<RecipeImport, AppError> {
    let (url, html) = fetch_html(raw_url).await?;
    let parsed = parse_recipe(&html).ok_or_else(|| {
        AppError::Unprocessable(
            "Couldn't find a recipe on that page. Only pages that publish structured recipe data can be imported."
                .to_string(),
        )
    })?;
    let ingredients = ingredient::repository::list(pool).await?;
    let units: Vec<Unit> = unit::repository::list(pool)
        .await?
        .into_iter()
        .map(|u| u.unit)
        .collect();
    Ok(build_import(parsed, url.to_string(), &ingredients, &units))
}

/// Turns a parsed page into dialog entries, matching each line to a saved ingredient by name.
pub fn build_import(
    parsed: ParsedRecipe,
    source_url: String,
    ingredients: &[Ingredient],
    units: &[Unit],
) -> RecipeImport {
    let mut by_name: HashMap<String, &Ingredient> = HashMap::new();
    for ingredient in ingredients {
        by_name
            .entry(match_key(&ingredient.name))
            .or_insert(ingredient);
    }

    let mut lines: Vec<ImportedIngredient> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    for text in &parsed.ingredient_lines {
        let line = parse_line(text);
        let key = match_key(&line.name);
        let imported = match by_name.get(&key) {
            Some(saved) => saved_ingredient(saved, &line, text),
            None => new_ingredient(&line, units),
        };
        // The same ingredient twice would collide on save, so the repeat folds into the first.
        match seen.get(&key) {
            Some(&first) => {
                let existing = &mut lines[first];
                let merged = match existing.note.take() {
                    Some(note) => format!("{note}; also {text}"),
                    None => format!("also {text}"),
                };
                existing.note = Some(merged);
            }
            None => {
                seen.insert(key, lines.len());
                lines.push(imported);
            }
        }
    }

    RecipeImport {
        title: parsed.title,
        time_mins: parsed.time_mins,
        servings: parsed.servings,
        source_url,
        ingredients: lines,
    }
}

/// A saved ingredient has one fixed unit, so an amount in a different unit can't be carried
/// over; the page's own wording goes in the note for the user to convert.
fn saved_ingredient(
    saved: &Ingredient,
    line: &super::line::ParsedLine,
    text: &str,
) -> ImportedIngredient {
    let same_unit = line
        .unit
        .as_ref()
        .is_none_or(|u| canonical_unit(u) == canonical_unit(&saved.unit));
    let (qty, note) = if same_unit {
        (line.qty, line.note.clone())
    } else {
        (None, Some(text.to_string()))
    };
    ImportedIngredient {
        name: saved.name.clone(),
        unit: saved.unit.clone(),
        qty,
        note,
        ingredient: IngredientEntry::Existing(saved.id),
    }
}

fn new_ingredient(line: &super::line::ParsedLine, units: &[Unit]) -> ImportedIngredient {
    let wanted = line.unit.as_deref().unwrap_or(COUNT_UNIT);
    let wanted_key = canonical_unit(wanted);
    let (label, entry) = match units.iter().find(|u| canonical_unit(&u.name) == wanted_key) {
        Some(saved) => (saved.name.clone(), UnitEntry::Existing(saved.id)),
        None => (wanted_key.clone(), UnitEntry::New(wanted_key)),
    };
    ImportedIngredient {
        name: line.name.clone(),
        unit: label,
        qty: line.qty,
        note: line.note.clone(),
        ingredient: IngredientEntry::New {
            name: line.name.clone(),
            purchase_size: None,
            unit: entry,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use time::OffsetDateTime;
    use uuid::Uuid;

    fn saved_unit(name: &str) -> Unit {
        Unit {
            id: Uuid::now_v7(),
            household_id: None,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
            name: name.to_string(),
            description: None,
        }
    }

    fn saved_ingredient_row(name: &str, unit: &Unit) -> Ingredient {
        Ingredient {
            id: Uuid::now_v7(),
            household_id: None,
            name: name.to_string(),
            purchase_size: None,
            unit: unit.name.clone(),
            unit_id: unit.id,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    fn recipe(lines: &[&str]) -> ParsedRecipe {
        ParsedRecipe {
            title: "Test".to_string(),
            time_mins: Some(30),
            servings: Some(4),
            ingredient_lines: lines.iter().map(|l| l.to_string()).collect(),
        }
    }

    fn import(lines: &[&str], ingredients: &[Ingredient], units: &[Unit]) -> RecipeImport {
        build_import(
            recipe(lines),
            "https://example.com/r".to_string(),
            ingredients,
            units,
        )
    }

    #[test]
    fn carries_the_recipe_details_across() {
        let result = import(&[], &[], &[]);
        assert_eq!(result.title, "Test");
        assert_eq!(result.time_mins, Some(30));
        assert_eq!(result.servings, Some(4));
        assert_eq!(result.source_url, "https://example.com/r");
    }

    #[test]
    fn matches_saved_ingredients_by_name_ignoring_case_and_plurals() {
        let each = saved_unit("each");
        let egg = saved_ingredient_row("Egg", &each);
        let result = import(&["2 eggs"], std::slice::from_ref(&egg), &[each]);
        let line = &result.ingredients[0];
        assert_eq!(line.ingredient, IngredientEntry::Existing(egg.id));
        assert_eq!(line.name, "Egg");
        assert_eq!(line.unit, "each");
        assert_eq!(line.qty, Some(Decimal::from(2)));
    }

    #[test]
    fn unmatched_lines_become_new_ingredients_with_a_saved_unit_when_there_is_one() {
        let g = saved_unit("g");
        let result = import(&["200 g rice noodles"], &[], std::slice::from_ref(&g));
        let line = &result.ingredients[0];
        assert_eq!(
            line.ingredient,
            IngredientEntry::New {
                name: "rice noodles".to_string(),
                purchase_size: None,
                unit: UnitEntry::Existing(g.id),
            }
        );
        assert_eq!(line.qty, Some(Decimal::from(200)));
    }

    #[test]
    fn unknown_units_are_offered_as_new_ones() {
        let result = import(&["2 cups flour", "3 eggs"], &[], &[]);
        let units: Vec<_> = result
            .ingredients
            .iter()
            .map(|l| match &l.ingredient {
                IngredientEntry::New { unit, .. } => unit.clone(),
                other => panic!("expected new, got {other:?}"),
            })
            .collect();
        assert_eq!(
            units,
            vec![UnitEntry::New("cup".into()), UnitEntry::New("each".into())]
        );
    }

    #[test]
    fn saved_units_match_across_spellings() {
        let tablespoon = saved_unit("Tablespoon");
        let result = import(
            &["2 tbsp soy sauce"],
            &[],
            std::slice::from_ref(&tablespoon),
        );
        assert!(matches!(
            &result.ingredients[0].ingredient,
            IngredientEntry::New { unit: UnitEntry::Existing(id), .. } if *id == tablespoon.id
        ));
        assert_eq!(result.ingredients[0].unit, "Tablespoon");
    }

    #[test]
    fn a_different_unit_for_a_saved_ingredient_keeps_the_wording_instead_of_the_amount() {
        let g = saved_unit("g");
        let flour = saved_ingredient_row("flour", &g);
        let result = import(&["2 cups flour"], std::slice::from_ref(&flour), &[g]);
        let line = &result.ingredients[0];
        assert_eq!(line.ingredient, IngredientEntry::Existing(flour.id));
        assert_eq!(line.qty, None);
        assert_eq!(line.note.as_deref(), Some("2 cups flour"));
    }

    #[test]
    fn the_same_ingredient_twice_is_folded_into_one_line() {
        let result = import(&["1 onion", "2 onions, sliced"], &[], &[]);
        assert_eq!(result.ingredients.len(), 1);
        assert_eq!(
            result.ingredients[0].note.as_deref(),
            Some("also 2 onions, sliced")
        );
    }
}
