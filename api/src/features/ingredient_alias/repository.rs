use crate::features::recipe_import::matching::phrase_key;
use domain::ingredient_alias::IngredientAlias;
use sqlx::{PgConnection, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<IngredientAlias>, sqlx::Error> {
    let aliases = sqlx::query_as!(
        IngredientAlias,
        r#"
        SELECT id, household_id, ingredient_id, phrase, created_at, updated_at
        FROM ingredient_alias
        ORDER BY phrase
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(aliases)
}

/// Every remembered phrase and the ingredient chosen for it, keyed by the normalised phrase.
pub async fn map(pool: &PgPool) -> Result<HashMap<String, Uuid>, sqlx::Error> {
    Ok(list(pool)
        .await?
        .into_iter()
        .filter(|a| a.household_id.is_none())
        .map(|a| (a.phrase, a.ingredient_id))
        .collect())
}

/// Remembers that `phrase` means `ingredient_id`, replacing any earlier choice for it. A phrase
/// that already is the ingredient's own name needs no alias; `None` is returned for it.
pub async fn remember(
    conn: &mut PgConnection,
    phrase: &str,
    ingredient_id: Uuid,
) -> Result<Option<IngredientAlias>, sqlx::Error> {
    let key = phrase_key(phrase);
    if key.is_empty() {
        return Ok(None);
    }
    let alias = sqlx::query_as!(
        IngredientAlias,
        r#"
        INSERT INTO ingredient_alias (household_id, ingredient_id, phrase)
        SELECT NULL, ingredient.id, $1::text
        FROM ingredient
        WHERE ingredient.id = $2 AND lower(btrim(ingredient.name)) <> $1::text
        ON CONFLICT ON CONSTRAINT uq_ingredient_alias_household_phrase
        DO UPDATE SET ingredient_id = EXCLUDED.ingredient_id
        RETURNING id, household_id, ingredient_id, phrase, created_at, updated_at
        "#,
        key,
        ingredient_id,
    )
    .fetch_optional(conn)
    .await?;
    Ok(alias)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{ingredient, recipe};
    use domain::ingredient::{IngredientEntry, NewIngredient};
    use domain::protein::ProteinEntry;
    use domain::recipe::NewRecipe;
    use domain::recipe_ingredient::RecipeIngredientEntry;
    use domain::unit::UnitEntry;

    async fn ingredient_named(pool: &PgPool, name: &str) -> Uuid {
        ingredient::repository::create_with_unit(
            pool,
            NewIngredient {
                household_id: None,
                name: name.to_string(),
                purchase_size: None,
                unit: Some(UnitEntry::New("each".to_string())),
            },
        )
        .await
        .unwrap()
        .id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_remembered_phrase_is_stored_normalised_and_listed(pool: PgPool) {
        let eggs = ingredient_named(&pool, "Eggs").await;
        let mut conn = pool.acquire().await.unwrap();
        let alias = remember(&mut conn, "  Free-Range Eggs ", eggs)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(alias.phrase, "free-range egg");
        assert_eq!(alias.ingredient_id, eggs);
        assert_eq!(map(&pool).await.unwrap().get("free-range egg"), Some(&eggs));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn remembering_a_phrase_again_replaces_the_earlier_choice(pool: PgPool) {
        let eggs = ingredient_named(&pool, "Eggs").await;
        let quail = ingredient_named(&pool, "Quail eggs").await;
        let mut conn = pool.acquire().await.unwrap();
        remember(&mut conn, "little eggs", eggs).await.unwrap();
        remember(&mut conn, "little eggs", quail).await.unwrap();
        let all = list(&pool).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].ingredient_id, quail);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_phrase_that_is_the_ingredients_own_name_is_not_stored(pool: PgPool) {
        let milk = ingredient_named(&pool, "milk").await;
        let mut conn = pool.acquire().await.unwrap();
        assert!(remember(&mut conn, "Milk", milk).await.unwrap().is_none());
        assert!(list(&pool).await.unwrap().is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn aliases_go_with_their_ingredient(pool: PgPool) {
        let eggs = ingredient_named(&pool, "Eggs").await;
        let mut conn = pool.acquire().await.unwrap();
        remember(&mut conn, "hen eggs", eggs).await.unwrap();
        drop(conn);
        ingredient::repository::delete(&pool, eggs).await.unwrap();
        assert!(list(&pool).await.unwrap().is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn saving_an_imported_recipe_remembers_each_lines_phrase(pool: PgPool) {
        let eggs = ingredient_named(&pool, "Eggs").await;
        let line = |ingredient, phrase: &str| RecipeIngredientEntry {
            ingredient,
            qty: None,
            note: None,
            is_optional: false,
            import_phrase: Some(phrase.to_string()),
        };
        let new_recipe = NewRecipe {
            household_id: None,
            title: "Omelette".to_string(),
            difficulty: 1,
            time_mins: 10,
            servings: 2,
            healthiness: None,
            min_gap_weeks: None,
            max_gap_weeks: None,
            protein: ProteinEntry::New("Egg".to_string()),
            cuisine: None,
            ingredients: vec![
                line(IngredientEntry::Existing(eggs), "large eggs"),
                line(
                    IngredientEntry::New {
                        name: "Chives".to_string(),
                        purchase_size: None,
                        unit: Some(UnitEntry::New("bunch".to_string())),
                    },
                    "snipped chive",
                ),
            ],
            tags: vec![],
        };
        recipe::repository::create_with_ingredients(&pool, new_recipe)
            .await
            .unwrap();

        let remembered = map(&pool).await.unwrap();
        assert_eq!(remembered.get("egg"), Some(&eggs));
        assert_eq!(remembered.len(), 2);
        assert!(remembered.contains_key("snipped chive"));
    }
}
