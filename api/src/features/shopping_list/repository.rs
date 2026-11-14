use crate::features::{meal_plan, shopping_list_item};
use domain::shopping_list::{
    NewShoppingList, ShoppingList, ShoppingListDetail, ShoppingListForPlan, UpdateShoppingList,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<ShoppingList>, sqlx::Error> {
    let lists = sqlx::query_as!(
        ShoppingList,
        r#"
        SELECT id, household_id, meal_plan_id, name, created_at, updated_at
        FROM shopping_list
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(lists)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<ShoppingList, sqlx::Error> {
    let list = sqlx::query_as!(
        ShoppingList,
        r#"
        SELECT id, household_id, meal_plan_id, name, created_at, updated_at
        FROM shopping_list
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(list)
}

/// The list generated from a plan, if one has been made.
pub async fn find_for_plan(
    pool: &PgPool,
    meal_plan_id: Uuid,
) -> Result<Option<ShoppingList>, sqlx::Error> {
    sqlx::query_as!(
        ShoppingList,
        r#"
        SELECT id, household_id, meal_plan_id, name, created_at, updated_at
        FROM shopping_list
        WHERE meal_plan_id = $1
        "#,
        meal_plan_id,
    )
    .fetch_optional(pool)
    .await
}

/// Build a plan's list, or rebuild the one it already has. Each line is one
/// ingredient summed across the plan's recipes, in the ingredient's current
/// unit. Rebuilding keeps the purchased tick on lines that are still needed,
/// drops lines that no longer are, leaves free-text lines alone, and bumps the list's `updated_at`, which is
/// what the stale check compares against.
pub async fn generate_for_plan(
    pool: &PgPool,
    meal_plan_id: Uuid,
) -> Result<ShoppingList, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let list = sqlx::query_as!(
        ShoppingList,
        r#"
        INSERT INTO shopping_list (household_id, meal_plan_id)
        SELECT household_id, id
        FROM meal_plan
        WHERE id = $1
        ON CONFLICT ON CONSTRAINT uq_shopping_list_meal_plan
        DO UPDATE SET household_id = EXCLUDED.household_id
        RETURNING id, household_id, meal_plan_id, name, created_at, updated_at
        "#,
        meal_plan_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    let _removed = sqlx::query_scalar!(
        r#"
        DELETE FROM shopping_list_item
        WHERE shopping_list_id = $1
          AND shopping_list_item.ingredient_id IS NOT NULL
          AND NOT EXISTS (
              SELECT 1
              FROM planned_meal
              JOIN recipe_ingredient ON recipe_ingredient.recipe_id = planned_meal.recipe_id
              WHERE planned_meal.meal_plan_id = $2
                AND recipe_ingredient.ingredient_id = shopping_list_item.ingredient_id
          )
        RETURNING id
        "#,
        list.id,
        meal_plan_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    let _written = sqlx::query_scalar!(
        r#"
        INSERT INTO shopping_list_item (shopping_list_id, ingredient_id, unit_id, qty, note)
        SELECT $1::uuid, needed.ingredient_id, ingredient.unit_id, needed.qty,
               CASE WHEN needed.all_optional THEN 'optional' END
        FROM (
            SELECT recipe_ingredient.ingredient_id,
                   SUM(recipe_ingredient.qty) AS qty,
                   BOOL_AND(recipe_ingredient.is_optional) AS all_optional
            FROM planned_meal
            JOIN recipe_ingredient ON recipe_ingredient.recipe_id = planned_meal.recipe_id
            WHERE planned_meal.meal_plan_id = $2
            GROUP BY recipe_ingredient.ingredient_id
        ) AS needed
        JOIN ingredient ON ingredient.id = needed.ingredient_id
        ON CONFLICT ON CONSTRAINT uq_shopping_list_item
        DO UPDATE SET unit_id = EXCLUDED.unit_id,
                      qty = EXCLUDED.qty,
                      note = EXCLUDED.note
        RETURNING id
        "#,
        list.id,
        meal_plan_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(list)
}

/// The plan's list, generating it first if the plan has none yet.
pub async fn ensure_for_plan(
    pool: &PgPool,
    meal_plan_id: Uuid,
) -> Result<ShoppingList, sqlx::Error> {
    match find_for_plan(pool, meal_plan_id).await? {
        Some(list) => Ok(list),
        None => generate_for_plan(pool, meal_plan_id).await,
    }
}

/// Whether any day of the list's plan changed after the list was generated.
async fn is_stale(pool: &PgPool, list_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COALESCE(MAX(planned_meal.updated_at) > shopping_list.updated_at, false) AS "is_stale!"
        FROM shopping_list
        LEFT JOIN planned_meal ON planned_meal.meal_plan_id = shopping_list.meal_plan_id
        WHERE shopping_list.id = $1
        GROUP BY shopping_list.id
        "#,
        list_id,
    )
    .fetch_one(pool)
    .await
}

/// A plan and its list with every line, ready for the shopping-list page.
pub async fn for_plan(
    pool: &PgPool,
    meal_plan_id: Uuid,
) -> Result<ShoppingListForPlan, sqlx::Error> {
    let plan = meal_plan::repository::get_summary(pool, meal_plan_id).await?;
    let list = match find_for_plan(pool, plan.id).await? {
        Some(list) => {
            let items = shopping_list_item::repository::lines(pool, list.id).await?;
            let is_stale = is_stale(pool, list.id).await?;
            Some(ShoppingListDetail {
                list,
                items,
                is_stale,
            })
        }
        None => None,
    };
    Ok(ShoppingListForPlan { plan, list })
}

/// [`for_plan`] for the latest active plan. `None` when there is no such plan.
pub async fn for_latest_plan(pool: &PgPool) -> Result<Option<ShoppingListForPlan>, sqlx::Error> {
    let Some(plan) = meal_plan::repository::latest(pool).await? else {
        return Ok(None);
    };
    for_plan(pool, plan.id).await.map(Some)
}

pub async fn create(pool: &PgPool, new_list: NewShoppingList) -> Result<ShoppingList, sqlx::Error> {
    let list = sqlx::query_as!(
        ShoppingList,
        r#"
        INSERT INTO shopping_list (household_id, meal_plan_id, name)
        VALUES ($1, $2, $3)
        RETURNING id, household_id, meal_plan_id, name, created_at, updated_at
        "#,
        new_list.household_id,
        new_list.meal_plan_id,
        new_list.name,
    )
    .fetch_one(pool)
    .await?;
    Ok(list)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    list: UpdateShoppingList,
) -> Result<ShoppingList, sqlx::Error> {
    let list = sqlx::query_as!(
        ShoppingList,
        r#"
        UPDATE shopping_list
        SET meal_plan_id = COALESCE($1, meal_plan_id),
            name = COALESCE($2, name)
        WHERE id = $3
        RETURNING id, household_id, meal_plan_id, name, created_at, updated_at
        "#,
        list.meal_plan_id,
        list.name,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(list)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<ShoppingList, sqlx::Error> {
    let list = sqlx::query_as!(
        ShoppingList,
        r#"
        DELETE FROM shopping_list
        WHERE id = $1
        RETURNING id, household_id, meal_plan_id, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::shopping_list_item::repository::{
        add_custom, clear_bought, lines, remove_custom, set_purchased,
    };

    struct Fixture {
        plan_id: Uuid,
        rice_recipe: Uuid,
    }

    async fn fixture(pool: &PgPool) -> Fixture {
        let household =
            sqlx::query_scalar!("INSERT INTO household (name) VALUES ('Home') RETURNING id")
                .fetch_one(pool)
                .await
                .unwrap();
        let unit = sqlx::query_scalar!("INSERT INTO unit (name) VALUES ('g') RETURNING id")
            .fetch_one(pool)
            .await
            .unwrap();
        let mut recipes = Vec::new();
        for (title, name) in [("Pasta", "Spaghetti"), ("Rice bowl", "Rice")] {
            let ingredient = sqlx::query_scalar!(
                "INSERT INTO ingredient (unit_id, name) VALUES ($1, $2) RETURNING id",
                unit,
                name,
            )
            .fetch_one(pool)
            .await
            .unwrap();
            let recipe = sqlx::query_scalar!(
                "INSERT INTO recipe (title, difficulty, time_mins, servings) VALUES ($1, 1, 20, 2) RETURNING id",
                title,
            )
            .fetch_one(pool)
            .await
            .unwrap();
            sqlx::query_scalar!(
                "INSERT INTO recipe_ingredient (recipe_id, ingredient_id, qty) VALUES ($1, $2, 100) RETURNING id",
                recipe,
                ingredient,
            )
            .fetch_one(pool)
            .await
            .unwrap();
            recipes.push(recipe);
        }
        let plan_id = sqlx::query_scalar!(
            "INSERT INTO meal_plan (household_id, start_date) VALUES ($1, '2026-11-02') RETURNING id",
            household,
        )
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "INSERT INTO planned_meal (meal_plan_id, recipe_id, day_index, scheduled_date)
             VALUES ($1, $2, 1, '2026-11-02'), ($1, $3, 2, '2026-11-03')
             RETURNING id",
            plan_id,
            recipes[0],
            recipes[1],
        )
        .fetch_all(pool)
        .await
        .unwrap();
        Fixture {
            plan_id,
            rice_recipe: recipes[1],
        }
    }

    fn names(lines: &[domain::shopping_list_item::ShoppingListLine]) -> Vec<&str> {
        lines.iter().map(|l| l.ingredient.as_str()).collect()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn rebuild_keeps_custom_items_and_their_tick(pool: PgPool) {
        let f = fixture(&pool).await;
        let list = generate_for_plan(&pool, f.plan_id).await.unwrap();
        let bags = add_custom(&pool, list.id, "  Bin bags ").await.unwrap();
        sqlx::query_scalar!(
            "UPDATE shopping_list_item SET is_purchased = true WHERE id = $1 RETURNING id",
            bags.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "DELETE FROM planned_meal WHERE recipe_id = $1 RETURNING id",
            f.rice_recipe,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        generate_for_plan(&pool, f.plan_id).await.unwrap();

        let after = lines(&pool, list.id).await.unwrap();
        assert_eq!(names(&after), ["Spaghetti", "Bin bags"]);
        let kept = &after[1];
        assert_eq!(kept.id, bags.id);
        assert!(kept.is_purchased);
        assert!(kept.ingredient_id.is_none());
        assert!(kept.qty.is_none());
        assert!(kept.used_in.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn rebuild_with_an_emptied_plan_still_keeps_custom_items(pool: PgPool) {
        let f = fixture(&pool).await;
        let list = generate_for_plan(&pool, f.plan_id).await.unwrap();
        add_custom(&pool, list.id, "Bin bags").await.unwrap();
        sqlx::query_scalar!(
            "DELETE FROM planned_meal WHERE meal_plan_id = $1 RETURNING id",
            f.plan_id,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        generate_for_plan(&pool, f.plan_id).await.unwrap();

        let after = lines(&pool, list.id).await.unwrap();
        assert_eq!(names(&after), ["Bin bags"]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn custom_items_can_repeat_and_be_removed_but_blank_ones_are_refused(pool: PgPool) {
        let f = fixture(&pool).await;
        let list = generate_for_plan(&pool, f.plan_id).await.unwrap();

        let first = add_custom(&pool, list.id, "Bin bags").await.unwrap();
        add_custom(&pool, list.id, "Bin bags").await.unwrap();
        assert!(add_custom(&pool, list.id, "   ").await.is_err());
        assert_eq!(lines(&pool, list.id).await.unwrap().len(), 4);

        remove_custom(&pool, list.id, first.id).await.unwrap();
        let after = lines(&pool, list.id).await.unwrap();
        assert_eq!(names(&after), ["Rice", "Spaghetti", "Bin bags"]);

        let ingredient_line = after[0].id;
        assert!(remove_custom(&pool, list.id, ingredient_line)
            .await
            .is_err());
        assert_eq!(lines(&pool, list.id).await.unwrap().len(), 3);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn clearing_bought_removes_only_ticked_custom_items(pool: PgPool) {
        let f = fixture(&pool).await;
        let list = generate_for_plan(&pool, f.plan_id).await.unwrap();
        let bags = add_custom(&pool, list.id, "Bin bags").await.unwrap();
        add_custom(&pool, list.id, "Candles").await.unwrap();
        let before = lines(&pool, list.id).await.unwrap();
        set_purchased(&pool, before[0].id, true).await.unwrap();
        set_purchased(&pool, bags.id, true).await.unwrap();

        assert_eq!(clear_bought(&pool, list.id).await.unwrap(), 2);

        let after = lines(&pool, list.id).await.unwrap();
        assert_eq!(names(&after), ["Spaghetti", "Candles"]);

        generate_for_plan(&pool, f.plan_id).await.unwrap();
        let rebuilt = lines(&pool, list.id).await.unwrap();
        assert_eq!(names(&rebuilt), ["Spaghetti", "Candles"]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_unitless_ingredient_gets_a_line_with_no_unit(pool: PgPool) {
        let household =
            sqlx::query_scalar!("INSERT INTO household (name) VALUES ('Home') RETURNING id")
                .fetch_one(&pool)
                .await
                .unwrap();
        let eggs =
            sqlx::query_scalar!("INSERT INTO ingredient (name) VALUES ('Eggs') RETURNING id")
                .fetch_one(&pool)
                .await
                .unwrap();
        let recipe = sqlx::query_scalar!(
            "INSERT INTO recipe (title, difficulty, time_mins, servings) VALUES ('Omelette', 1, 10, 2) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "INSERT INTO recipe_ingredient (recipe_id, ingredient_id, qty) VALUES ($1, $2, 3) RETURNING id",
            recipe,
            eggs,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let plan = sqlx::query_scalar!(
            "INSERT INTO meal_plan (household_id, start_date) VALUES ($1, '2026-11-02') RETURNING id",
            household,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "INSERT INTO planned_meal (meal_plan_id, recipe_id, day_index, scheduled_date)
             VALUES ($1, $2, 1, '2026-11-02')
             RETURNING id",
            plan,
            recipe,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let list = generate_for_plan(&pool, plan).await.unwrap();

        let items = sqlx::query_scalar!(
            "SELECT unit_id FROM shopping_list_item WHERE shopping_list_id = $1",
            list.id,
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(items, vec![None]);
        let lines = lines(&pool, list.id).await.unwrap();
        assert_eq!(names(&lines), vec!["Eggs"]);
        assert_eq!(lines[0].unit, "");
    }
}
