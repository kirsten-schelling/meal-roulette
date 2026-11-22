use super::fetch::fetch_html;
use super::jsonld::{parse_recipe, ParsedRecipe};
use super::line::{canonical_unit, parse_line, ParsedLine};
use super::matching::{
    capitalise_first, clean_phrase, convert_amount, find_match, phrase_key, Amount, CONFIDENT,
};
use crate::error::AppError;
use crate::features::{ingredient, ingredient_alias, unit};
use domain::ingredient::{Ingredient, IngredientEntry};
use domain::recipe_import::{ImportedIngredient, ImportedLine, MatchSuggestion, RecipeImport};
use domain::unit::{Unit, UnitEntry};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

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
    let aliases = ingredient_alias::repository::map(pool).await?;
    Ok(build_import(
        parsed,
        url.to_string(),
        &ingredients,
        &units,
        &aliases,
    ))
}

/// Turns a parsed page into dialog entries. Each line is resolved against the library and as a
/// new ingredient, and starts as the library match only when that match is confident.
pub fn build_import(
    parsed: ParsedRecipe,
    source_url: String,
    ingredients: &[Ingredient],
    units: &[Unit],
    aliases: &HashMap<String, Uuid>,
) -> RecipeImport {
    let names: Vec<(Uuid, &str)> = ingredients
        .iter()
        .map(|i| (i.id, i.name.as_str()))
        .collect();

    let mut lines: Vec<ImportedIngredient> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    for text in &parsed.ingredient_lines {
        let line = parse_line(text);
        let phrase = clean_phrase(&line.name);
        let suggestion = find_match(&phrase, &names, aliases).map(|found| {
            let saved = &ingredients[found.index];
            MatchSuggestion {
                confidence: found.confidence,
                kind: found.kind,
                line: saved_line(saved, &line, text),
            }
        });
        let use_suggestion = suggestion
            .as_ref()
            .is_some_and(|s| s.confidence >= CONFIDENT);
        let key = match (&suggestion, use_suggestion) {
            (Some(s), true) => format!("{:?}", s.line.ingredient),
            _ => phrase_key(&phrase),
        };
        let imported = ImportedIngredient {
            as_new: new_line(&phrase, &line, units),
            phrase,
            suggestion,
            use_suggestion,
        };
        // The same ingredient twice would collide on save, so the repeat folds into the first.
        match seen.get(&key) {
            Some(&first) => fold_note(&mut lines[first], text),
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

fn fold_note(first: &mut ImportedIngredient, text: &str) {
    let mut lines = vec![&mut first.as_new];
    if let Some(suggestion) = first.suggestion.as_mut() {
        lines.push(&mut suggestion.line);
    }
    for line in lines {
        line.note = Some(match line.note.take() {
            Some(note) => format!("{note}; also {text}"),
            None => format!("also {text}"),
        });
    }
}

/// A saved ingredient has one fixed unit. The amount is converted when the page's unit is of
/// the same kind (ml to l, tbsp to ml); otherwise the page's own wording goes in the note for
/// the user to convert.
fn saved_line(saved: &Ingredient, line: &ParsedLine, text: &str) -> ImportedLine {
    let (qty, note) = match convert_amount(line.qty, line.unit.as_deref(), &saved.unit) {
        Amount::Keep(qty) => (qty, line.note.clone()),
        Amount::Converted(qty) => (Some(qty), line.note.clone()),
        Amount::Unrelated => (None, Some(text.to_string())),
    };
    ImportedLine {
        name: saved.name.clone(),
        unit: saved.unit.clone(),
        qty,
        note,
        ingredient: IngredientEntry::Existing(saved.id),
    }
}

fn new_line(phrase: &str, line: &ParsedLine, units: &[Unit]) -> ImportedLine {
    let (label, entry) = match line.unit.as_deref().map(canonical_unit) {
        None => (String::new(), None),
        Some(key) => match units.iter().find(|u| canonical_unit(&u.name) == key) {
            Some(saved) => (saved.name.clone(), Some(UnitEntry::Existing(saved.id))),
            None => (key.clone(), Some(UnitEntry::New(key))),
        },
    };
    let name = capitalise_first(phrase);
    ImportedLine {
        name: name.clone(),
        unit: label,
        qty: line.qty,
        note: line.note.clone(),
        ingredient: IngredientEntry::New {
            name,
            purchase_size: None,
            unit: entry,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::recipe_import::MatchKind;
    use rust_decimal::Decimal;
    use time::OffsetDateTime;

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
            unit_id: Some(unit.id),
            shopping_category_id: None,
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

    fn import_with(
        lines: &[&str],
        ingredients: &[Ingredient],
        units: &[Unit],
        aliases: &HashMap<String, Uuid>,
    ) -> RecipeImport {
        build_import(
            recipe(lines),
            "https://example.com/r".to_string(),
            ingredients,
            units,
            aliases,
        )
    }

    fn import(lines: &[&str], ingredients: &[Ingredient], units: &[Unit]) -> RecipeImport {
        import_with(lines, ingredients, units, &HashMap::new())
    }

    fn chosen(line: &ImportedIngredient) -> &ImportedLine {
        match (&line.suggestion, line.use_suggestion) {
            (Some(s), true) => &s.line,
            _ => &line.as_new,
        }
    }

    fn dec(n: &str) -> Option<Decimal> {
        Some(n.parse().unwrap())
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
    fn unmatched_lines_become_new_ingredients_with_a_saved_unit_when_there_is_one() {
        let g = saved_unit("g");
        let result = import(&["200 g rice noodles"], &[], std::slice::from_ref(&g));
        let line = &result.ingredients[0];
        assert!(line.suggestion.is_none());
        assert_eq!(
            line.as_new.ingredient,
            IngredientEntry::New {
                name: "Rice noodles".to_string(),
                purchase_size: None,
                unit: Some(UnitEntry::Existing(g.id)),
            }
        );
        assert_eq!(line.as_new.qty, dec("200"));
    }

    #[test]
    fn a_new_count_line_has_no_unit() {
        let result = import(&["3 eggs"], &[], &[]);
        let line = &result.ingredients[0].as_new;
        assert_eq!(line.unit, "");
        assert_eq!(line.qty, dec("3"));
        assert!(matches!(
            line.ingredient,
            IngredientEntry::New { unit: None, .. }
        ));
    }

    #[test]
    fn unknown_units_are_offered_as_new_ones() {
        let result = import(&["2 cups flour", "3 bunches parsley"], &[], &[]);
        let units: Vec<_> = result
            .ingredients
            .iter()
            .map(|l| match &l.as_new.ingredient {
                IngredientEntry::New { unit, .. } => unit.clone().unwrap(),
                other => panic!("expected new, got {other:?}"),
            })
            .collect();
        assert_eq!(
            units,
            vec![UnitEntry::New("cup".into()), UnitEntry::New("bunch".into())]
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
        let line = &result.ingredients[0].as_new;
        assert!(matches!(
            &line.ingredient,
            IngredientEntry::New { unit: Some(UnitEntry::Existing(id)), .. } if *id == tablespoon.id
        ));
        assert_eq!(line.unit, "Tablespoon");
    }

    #[test]
    fn an_unrelated_unit_for_a_saved_ingredient_keeps_the_wording_instead_of_the_amount() {
        let g = saved_unit("g");
        let flour = saved_ingredient_row("Flour", &g);
        let result = import(&["2 cups flour"], std::slice::from_ref(&flour), &[g]);
        let line = chosen(&result.ingredients[0]);
        assert_eq!(line.ingredient, IngredientEntry::Existing(flour.id));
        assert_eq!(line.qty, None);
        assert_eq!(line.note.as_deref(), Some("2 cups flour"));
    }

    #[test]
    fn the_same_ingredient_twice_is_folded_into_one_line() {
        let result = import(&["1 onion", "2 onions, sliced"], &[], &[]);
        assert_eq!(result.ingredients.len(), 1);
        assert_eq!(
            result.ingredients[0].as_new.note.as_deref(),
            Some("also 2 onions, sliced")
        );
    }

    #[test]
    fn two_phrasings_of_one_library_ingredient_fold_together() {
        let each = saved_unit("each");
        let lemon = saved_ingredient_row("Lemon", &each);
        let result = import(
            &["1 lemon", "lemon wedges, to serve"],
            std::slice::from_ref(&lemon),
            &[each],
        );
        assert_eq!(result.ingredients.len(), 1);
    }

    struct Library {
        ingredients: Vec<Ingredient>,
        units: Vec<Unit>,
    }

    fn library() -> Library {
        let each = saved_unit("each");
        let g = saved_unit("g");
        let ml = saved_unit("ml");
        let l = saved_unit("l");
        let ingredients = vec![
            saved_ingredient_row("Eggs", &each),
            saved_ingredient_row("Milk", &l),
            saved_ingredient_row("Lemon", &each),
            saved_ingredient_row("Plain flour", &g),
            saved_ingredient_row("Butter", &g),
            saved_ingredient_row("Olive oil", &ml),
            saved_ingredient_row("Garlic", &each),
            saved_ingredient_row("Onion", &each),
            saved_ingredient_row("Caster sugar", &g),
            saved_ingredient_row("Tomatoes", &each),
            saved_ingredient_row("Chicken breast", &g),
            saved_ingredient_row("Soy sauce", &ml),
            saved_ingredient_row("Parsley", &g),
            saved_ingredient_row("Cheddar", &g),
            saved_ingredient_row("Carrots", &each),
        ];
        Library {
            ingredients,
            units: vec![each, g, ml, l],
        }
    }

    /// (line, library ingredient it should land on or None for a new one, qty, unit label)
    type Case = (
        &'static str,
        Option<&'static str>,
        Option<&'static str>,
        &'static str,
    );

    #[test]
    fn real_lines_land_on_the_right_ingredient_with_the_right_amount() {
        let cases: &[Case] = &[
            ("large eggs", Some("Eggs"), None, "each"),
            ("2 large eggs", Some("Eggs"), Some("2"), "each"),
            ("300ml milk", Some("Milk"), Some("0.3"), "l"),
            ("300 ml whole milk", None, Some("300"), "ml"),
            ("lemon wedges", Some("Lemon"), None, "each"),
            ("lemon wedges, to serve", Some("Lemon"), None, "each"),
            ("1 lemon, juiced", Some("Lemon"), Some("1"), "each"),
            ("1½ cups plain flour", Some("Plain flour"), None, "g"),
            (
                "250g plain flour, sifted",
                Some("Plain flour"),
                Some("250"),
                "g",
            ),
            ("0.5 kg butter", Some("Butter"), Some("500"), "g"),
            ("2 tbsp olive oil", Some("Olive oil"), Some("30"), "ml"),
            (
                "1 tablespoon extra virgin olive oil",
                Some("Olive oil"),
                Some("15"),
                "ml",
            ),
            ("3 cloves garlic, minced", Some("Garlic"), None, "each"),
            ("2 garlic cloves", Some("Garlic"), Some("2"), "each"),
            (
                "1 large onion, finely diced",
                Some("Onion"),
                Some("1"),
                "each",
            ),
            (
                "2 medium onions (about 300g)",
                Some("Onion"),
                Some("2"),
                "each",
            ),
            ("100g caster sugar", Some("Caster sugar"), Some("100"), "g"),
            (
                "4 ripe tomatoes, chopped",
                Some("Tomatoes"),
                Some("4"),
                "each",
            ),
            ("1 tomato", Some("Tomatoes"), Some("1"), "each"),
            (
                "500 g boneless skinless chicken breasts",
                Some("Chicken breast"),
                Some("500"),
                "g",
            ),
            ("2 tsp soy sauce", Some("Soy sauce"), Some("10"), "ml"),
            ("a handful of fresh parsley", Some("Parsley"), None, "g"),
            ("2-3 tbsp chopped fresh parsley", Some("Parsley"), None, "g"),
            ("150g mature cheddar, grated", None, Some("150"), "g"),
            (
                "3 carrots, peeled and sliced",
                Some("Carrots"),
                Some("3"),
                "each",
            ),
            ("1/2 tsp smoked paprika", None, Some("0.5"), "tsp"),
            ("½ cup milk", Some("Milk"), Some("0.12"), "l"),
            ("200 g rice noodles", None, Some("200"), "g"),
            (
                "1 (400g) can chopped tomatoes",
                Some("Tomatoes"),
                None,
                "each",
            ),
            ("pinch of salt", None, None, "pinch"),
        ];
        let lib = library();
        for (text, want, qty, unit) in cases {
            let result = import(&[text], &lib.ingredients, &lib.units);
            let line = chosen(&result.ingredients[0]);
            match want {
                Some(name) => {
                    assert_eq!(&line.name, name, "{text}");
                    assert!(
                        matches!(line.ingredient, IngredientEntry::Existing(_)),
                        "{text}"
                    );
                }
                None => assert!(
                    matches!(line.ingredient, IngredientEntry::New { .. }),
                    "{text}: {line:?}"
                ),
            }
            assert_eq!(&line.unit, unit, "{text}");
            if let Some(qty) = qty {
                assert_eq!(line.qty, dec(qty), "{text}");
            }
        }
    }

    #[test]
    fn the_three_named_lines_match_eggs_milk_and_lemon() {
        let lib = library();
        let result = import(
            &["large eggs", "300ml milk", "lemon wedges"],
            &lib.ingredients,
            &lib.units,
        );
        let names: Vec<_> = result
            .ingredients
            .iter()
            .map(|l| chosen(l).name.as_str())
            .collect();
        assert_eq!(names, ["Eggs", "Milk", "Lemon"]);
        assert_eq!(chosen(&result.ingredients[1]).qty, dec("0.3"));
    }

    #[test]
    fn a_confident_match_starts_chosen_and_offers_the_new_alternative() {
        let lib = library();
        let result = import(&["large eggs"], &lib.ingredients, &lib.units);
        let line = &result.ingredients[0];
        assert!(line.use_suggestion);
        assert_eq!(line.phrase, "eggs");
        let suggestion = line.suggestion.as_ref().unwrap();
        assert_eq!(suggestion.kind, MatchKind::Exact);
        assert_eq!(line.as_new.name, "Eggs");
    }

    #[test]
    fn a_weak_match_starts_as_new_but_is_still_offered() {
        let lib = library();
        let result = import(&["1 tbsp oil"], &lib.ingredients, &lib.units);
        let line = &result.ingredients[0];
        assert!(!line.use_suggestion);
        let suggestion = line.suggestion.as_ref().unwrap();
        assert_eq!(suggestion.line.name, "Olive oil");
        assert!(suggestion.confidence < CONFIDENT);
    }

    #[test]
    fn a_misspelling_within_one_edit_still_matches() {
        let lib = library();
        let result = import(&["3 carots"], &lib.ingredients, &lib.units);
        assert_eq!(chosen(&result.ingredients[0]).name, "Carrots");
    }

    #[test]
    fn a_remembered_phrase_wins_over_a_weak_guess() {
        let lib = library();
        let cheddar = lib
            .ingredients
            .iter()
            .find(|i| i.name == "Cheddar")
            .unwrap();
        let aliases = HashMap::from([(phrase_key("sharp white cheese"), cheddar.id)]);
        let result = import_with(
            &["200g sharp white cheese"],
            &lib.ingredients,
            &lib.units,
            &aliases,
        );
        let line = &result.ingredients[0];
        assert!(line.use_suggestion);
        assert_eq!(
            line.suggestion.as_ref().unwrap().kind,
            MatchKind::Remembered
        );
        assert_eq!(chosen(line).name, "Cheddar");
    }

    #[test]
    fn a_remembered_phrase_for_a_deleted_ingredient_is_ignored() {
        let lib = library();
        let aliases = HashMap::from([(phrase_key("sharp white cheese"), Uuid::now_v7())]);
        let result = import_with(
            &["200g sharp white cheese"],
            &lib.ingredients,
            &lib.units,
            &aliases,
        );
        assert!(!result.ingredients[0].use_suggestion);
    }
}
