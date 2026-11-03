use domain::shopping_list_item::{
    NewShoppingListItem, ShoppingListItem, ShoppingListLine, UpdateShoppingListItem,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<ShoppingListItem>, sqlx::Error> {
    let items = sqlx::query_as!(
        ShoppingListItem,
        r#"
        SELECT id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        FROM shopping_list_item
        ORDER BY created_at
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(items)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<ShoppingListItem, sqlx::Error> {
    let item = sqlx::query_as!(
        ShoppingListItem,
        r#"
        SELECT id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        FROM shopping_list_item
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

/// Every line of one list as the page shows it. A line counts as asleep while
/// the list's household has it slept until today or later; asleep lines sort
/// after the rest, and free-text lines after the ingredient lines.
pub async fn lines(
    pool: &PgPool,
    shopping_list_id: Uuid,
) -> Result<Vec<ShoppingListLine>, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListLine,
        r#"
        SELECT shopping_list_item.id, shopping_list_item.ingredient_id,
               COALESCE(ingredient.name, shopping_list_item.name) AS "ingredient!",
               shopping_list_item.qty, COALESCE(unit.name, '') AS "unit!", ingredient.purchase_size,
               CEIL(shopping_list_item.qty / ingredient.purchase_size)::int AS "packs?",
               shopping_list_item.is_purchased, shopping_list_item.note,
               ARRAY(
                   SELECT recipe.title
                   FROM planned_meal
                   JOIN recipe ON recipe.id = planned_meal.recipe_id
                   JOIN recipe_ingredient ON recipe_ingredient.recipe_id = recipe.id
                   WHERE planned_meal.meal_plan_id = shopping_list.meal_plan_id
                     AND recipe_ingredient.ingredient_id = shopping_list_item.ingredient_id
                   ORDER BY planned_meal.scheduled_date
               ) AS "used_in!",
               slept_ingredient.sleep_until AS "sleep_until?"
        FROM shopping_list_item
        JOIN shopping_list ON shopping_list.id = shopping_list_item.shopping_list_id
        LEFT JOIN ingredient ON ingredient.id = shopping_list_item.ingredient_id
        LEFT JOIN unit ON unit.id = shopping_list_item.unit_id
        LEFT JOIN slept_ingredient
               ON slept_ingredient.household_id = shopping_list.household_id
              AND slept_ingredient.ingredient_id = shopping_list_item.ingredient_id
              AND slept_ingredient.sleep_until >= CURRENT_DATE
        WHERE shopping_list_item.shopping_list_id = $1
        ORDER BY slept_ingredient.id IS NOT NULL,
                 shopping_list_item.ingredient_id IS NULL,
                 COALESCE(ingredient.name, shopping_list_item.name)
        "#,
        shopping_list_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn set_purchased(
    pool: &PgPool,
    id: Uuid,
    is_purchased: bool,
) -> Result<ShoppingListItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListItem,
        r#"
        UPDATE shopping_list_item
        SET is_purchased = $1
        WHERE id = $2
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        "#,
        is_purchased,
        id,
    )
    .fetch_one(pool)
    .await
}

/// Add a free-text line to a list. Rebuilding the list from its plan leaves it alone.
pub async fn add_custom(
    pool: &PgPool,
    shopping_list_id: Uuid,
    name: &str,
) -> Result<ShoppingListItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListItem,
        r#"
        INSERT INTO shopping_list_item (shopping_list_id, name)
        VALUES ($1, btrim($2))
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        "#,
        shopping_list_id,
        name,
    )
    .fetch_one(pool)
    .await
}

/// Remove a free-text line. An ingredient line is not found here: it goes when the
/// plan stops needing it.
pub async fn remove_custom(
    pool: &PgPool,
    shopping_list_id: Uuid,
    id: Uuid,
) -> Result<ShoppingListItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListItem,
        r#"
        DELETE FROM shopping_list_item
        WHERE id = $1 AND shopping_list_id = $2 AND ingredient_id IS NULL
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        "#,
        id,
        shopping_list_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    new_item: NewShoppingListItem,
) -> Result<ShoppingListItem, sqlx::Error> {
    let item = sqlx::query_as!(
        ShoppingListItem,
        r#"
        INSERT INTO shopping_list_item (shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        "#,
        new_item.shopping_list_id,
        new_item.ingredient_id,
        new_item.unit_id,
        new_item.qty,
        new_item.is_purchased,
        new_item.note,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    item: UpdateShoppingListItem,
) -> Result<ShoppingListItem, sqlx::Error> {
    let item = sqlx::query_as!(
        ShoppingListItem,
        r#"
        UPDATE shopping_list_item
        SET ingredient_id = COALESCE($1, ingredient_id),
            unit_id = COALESCE($2, unit_id),
            qty = COALESCE($3, qty),
            is_purchased = COALESCE($4, is_purchased),
            note = COALESCE($5, note)
        WHERE id = $6
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        "#,
        item.ingredient_id,
        item.unit_id,
        item.qty,
        item.is_purchased,
        item.note,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<ShoppingListItem, sqlx::Error> {
    let item = sqlx::query_as!(
        ShoppingListItem,
        r#"
        DELETE FROM shopping_list_item
        WHERE id = $1
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}
