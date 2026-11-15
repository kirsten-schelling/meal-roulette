use crate::features::household;
use domain::pantry::PantryLine;
use rust_decimal::Decimal;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

/// The counts of the household every unscoped request acts for, by ingredient name.
pub async fn list(pool: &PgPool) -> Result<Vec<PantryLine>, sqlx::Error> {
    let household_id = household::repository::default_household(pool).await?.id;
    sqlx::query_as!(
        PantryLine,
        r#"
        SELECT pantry_item.ingredient_id, ingredient.name AS ingredient,
               COALESCE(unit.name, '') AS "unit!", pantry_item.qty
        FROM pantry_item
        JOIN ingredient ON ingredient.id = pantry_item.ingredient_id
        LEFT JOIN unit ON unit.id = ingredient.unit_id
        WHERE pantry_item.household_id = $1
        ORDER BY ingredient.name
        "#,
        household_id,
    )
    .fetch_all(pool)
    .await
}

/// Overwrite an ingredient's count, creating it if the household had none. A
/// manual count always replaces the estimate.
pub async fn set_qty(
    pool: &PgPool,
    ingredient_id: Uuid,
    qty: Decimal,
) -> Result<PantryLine, sqlx::Error> {
    let household_id = household::repository::default_household(pool).await?.id;
    sqlx::query_as!(
        PantryLine,
        r#"
        WITH saved AS (
            INSERT INTO pantry_item (household_id, ingredient_id, qty)
            VALUES ($1, $2, $3)
            ON CONFLICT ON CONSTRAINT uq_pantry_item
            DO UPDATE SET qty = EXCLUDED.qty
            RETURNING ingredient_id, qty
        )
        SELECT saved.ingredient_id AS "ingredient_id!", ingredient.name AS "ingredient!",
               COALESCE(unit.name, '') AS "unit!", saved.qty AS "qty!"
        FROM saved
        JOIN ingredient ON ingredient.id = saved.ingredient_id
        LEFT JOIN unit ON unit.id = ingredient.unit_id
        "#,
        household_id,
        ingredient_id,
        qty,
    )
    .fetch_one(pool)
    .await
}

/// Take what a cooked recipe uses out of the counts the household already keeps,
/// never below 0. Ingredients with no count, optional lines and lines with no
/// amount are left alone.
pub async fn use_up_recipe(
    conn: &mut PgConnection,
    household_id: Uuid,
    recipe_id: Uuid,
) -> Result<(), sqlx::Error> {
    adjust_for_recipe(conn, household_id, recipe_id, Decimal::NEGATIVE_ONE).await
}

/// Undo [`use_up_recipe`] for a cook that was taken off the log. A count that was
/// held at 0 gets back the recipe's full amount, which the user can correct.
pub async fn give_back_recipe(
    conn: &mut PgConnection,
    household_id: Uuid,
    recipe_id: Uuid,
) -> Result<(), sqlx::Error> {
    adjust_for_recipe(conn, household_id, recipe_id, Decimal::ONE).await
}

async fn adjust_for_recipe(
    conn: &mut PgConnection,
    household_id: Uuid,
    recipe_id: Uuid,
    direction: Decimal,
) -> Result<(), sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        UPDATE pantry_item
        SET qty = GREATEST(pantry_item.qty + $3 * used.qty, 0)
        FROM (
            SELECT ingredient_id, SUM(qty) AS qty
            FROM recipe_ingredient
            WHERE recipe_id = $2 AND qty IS NOT NULL AND NOT is_optional
            GROUP BY ingredient_id
        ) AS used
        WHERE pantry_item.household_id = $1
          AND pantry_item.ingredient_id = used.ingredient_id
        RETURNING pantry_item.id
        "#,
        household_id,
        recipe_id,
        direction,
    )
    .fetch_all(conn)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    async fn ingredient_named(pool: &PgPool, name: &str) -> Uuid {
        let unit = sqlx::query_scalar!(
            "INSERT INTO unit (name) VALUES ('g') ON CONFLICT (household_id, name) DO UPDATE SET name = 'g' RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "INSERT INTO ingredient (unit_id, name) VALUES ($1, $2) RETURNING id",
            unit,
            name,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn recipe_using(pool: &PgPool, lines: &[(Uuid, Option<i64>, bool)]) -> Uuid {
        let recipe = sqlx::query_scalar!(
            "INSERT INTO recipe (title, difficulty, time_mins, servings) VALUES ('Dish', 1, 20, 2) RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        for (ingredient, qty, is_optional) in lines {
            sqlx::query_scalar!(
                "INSERT INTO recipe_ingredient (recipe_id, ingredient_id, qty, is_optional) VALUES ($1, $2, $3, $4) RETURNING id",
                recipe,
                ingredient,
                qty.map(Decimal::from),
                is_optional,
            )
            .fetch_one(pool)
            .await
            .unwrap();
        }
        recipe
    }

    fn qty_of(lines: &[PantryLine], ingredient: Uuid) -> Option<Decimal> {
        lines
            .iter()
            .find(|line| line.ingredient_id == ingredient)
            .map(|line| line.qty)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_count_is_created_then_overwritten_and_never_negative(pool: PgPool) {
        let rice = ingredient_named(&pool, "Rice").await;

        let created = set_qty(&pool, rice, Decimal::from(500)).await.unwrap();
        assert_eq!(created.ingredient, "Rice");
        assert_eq!(created.unit, "g");
        set_qty(&pool, rice, Decimal::new(125, 1)).await.unwrap();
        assert_eq!(
            qty_of(&list(&pool).await.unwrap(), rice),
            Some(Decimal::new(125, 1))
        );

        let refused = set_qty(&pool, rice, Decimal::from(-1)).await.unwrap_err();
        assert_eq!(
            AppError::from(refused).to_string(),
            "A pantry count cannot be below 0."
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn cooking_takes_the_recipes_amounts_down_to_at_least_zero(pool: PgPool) {
        let rice = ingredient_named(&pool, "Rice").await;
        let flour = ingredient_named(&pool, "Flour").await;
        let salt = ingredient_named(&pool, "Salt").await;
        let herbs = ingredient_named(&pool, "Herbs").await;
        let recipe = recipe_using(
            &pool,
            &[
                (rice, Some(200), false),
                (flour, Some(300), false),
                (salt, None, false),
                (herbs, Some(10), true),
            ],
        )
        .await;
        for (ingredient, qty) in [(rice, 500), (flour, 100), (salt, 50), (herbs, 50)] {
            set_qty(&pool, ingredient, Decimal::from(qty))
                .await
                .unwrap();
        }
        let household = household::repository::default_household(&pool)
            .await
            .unwrap()
            .id;

        use_up_recipe(&mut pool.acquire().await.unwrap(), household, recipe)
            .await
            .unwrap();

        let after = list(&pool).await.unwrap();
        assert_eq!(qty_of(&after, rice), Some(Decimal::from(300)));
        assert_eq!(qty_of(&after, flour), Some(Decimal::ZERO));
        assert_eq!(qty_of(&after, salt), Some(Decimal::from(50)));
        assert_eq!(qty_of(&after, herbs), Some(Decimal::from(50)));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn cooking_leaves_ingredients_without_a_count_uncounted_and_giving_back_restores(
        pool: PgPool,
    ) {
        let rice = ingredient_named(&pool, "Rice").await;
        let pasta = ingredient_named(&pool, "Pasta").await;
        let recipe = recipe_using(
            &pool,
            &[(rice, Some(200), false), (pasta, Some(100), false)],
        )
        .await;
        set_qty(&pool, rice, Decimal::from(500)).await.unwrap();
        let household = household::repository::default_household(&pool)
            .await
            .unwrap()
            .id;
        let mut conn = pool.acquire().await.unwrap();

        use_up_recipe(&mut conn, household, recipe).await.unwrap();
        give_back_recipe(&mut conn, household, recipe)
            .await
            .unwrap();

        let after = list(&pool).await.unwrap();
        assert_eq!(qty_of(&after, rice), Some(Decimal::from(500)));
        assert_eq!(qty_of(&after, pasta), None);
    }
}
