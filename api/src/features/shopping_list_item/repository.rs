use domain::shopping_list_item::{
    NewShoppingListItem, ShoppingListItem, ShoppingListLine, UpdateShoppingListItem,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<ShoppingListItem>, sqlx::Error> {
    let items = sqlx::query_as!(
        ShoppingListItem,
        r#"
        SELECT id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
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
        SELECT id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
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
/// after the rest, and lines that are not ingredients after the ingredient lines.
pub async fn lines(
    pool: &PgPool,
    shopping_list_id: Uuid,
) -> Result<Vec<ShoppingListLine>, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListLine,
        r#"
        SELECT shopping_list_item.id, shopping_list_item.ingredient_id,
               COALESCE(ingredient.name, shopping_item.name) AS "ingredient!",
               shopping_list_item.qty, COALESCE(unit.name, '') AS "unit!", ingredient.purchase_size,
               CEIL(shopping_list_item.qty / ingredient.purchase_size)::int AS "packs?",
               shopping_list_item.is_purchased, shopping_list_item.is_covered, shopping_list_item.note,
               ARRAY(
                   SELECT recipe.title
                   FROM planned_meal
                   JOIN recipe ON recipe.id = planned_meal.recipe_id
                   JOIN recipe_ingredient ON recipe_ingredient.recipe_id = recipe.id
                   WHERE planned_meal.meal_plan_id = shopping_list.meal_plan_id
                     AND recipe_ingredient.ingredient_id = shopping_list_item.ingredient_id
                   ORDER BY planned_meal.scheduled_date NULLS LAST, planned_meal.day_index
               ) AS "used_in!",
               COALESCE(ingredient.shopping_category_id, shopping_item.shopping_category_id) AS category_id,
               slept_ingredient.sleep_until AS "sleep_until?"
        FROM shopping_list_item
        JOIN shopping_list ON shopping_list.id = shopping_list_item.shopping_list_id
        LEFT JOIN ingredient ON ingredient.id = shopping_list_item.ingredient_id
        LEFT JOIN shopping_item ON shopping_item.id = shopping_list_item.shopping_item_id
        LEFT JOIN unit ON unit.id = shopping_list_item.unit_id
        LEFT JOIN slept_ingredient
               ON slept_ingredient.household_id = shopping_list.household_id
              AND slept_ingredient.ingredient_id = shopping_list_item.ingredient_id
              AND slept_ingredient.sleep_until >= CURRENT_DATE
        WHERE shopping_list_item.shopping_list_id = $1
          AND NOT shopping_list_item.is_cleared
        ORDER BY slept_ingredient.id IS NOT NULL,
                 shopping_list_item.ingredient_id IS NULL,
                 COALESCE(ingredient.name, shopping_item.name)
        "#,
        shopping_list_id,
    )
    .fetch_all(pool)
    .await
}

/// Tick or untick a line.
pub async fn set_purchased(
    pool: &PgPool,
    id: Uuid,
    is_purchased: bool,
) -> Result<ShoppingListItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListItem,
        r#"
        UPDATE shopping_list_item
        SET is_purchased = $2
        WHERE id = $1
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
        "#,
        id,
        is_purchased,
    )
    .fetch_one(pool)
    .await
}

/// Put a line the pantry covers onto the list, at its full amount.
pub async fn add_anyway(pool: &PgPool, id: Uuid) -> Result<ShoppingListItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListItem,
        r#"
        UPDATE shopping_list_item
        SET is_covered = false
        WHERE id = $1 AND is_covered
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await
}

/// Add a line for something that is not an ingredient. It is matched by name, ignoring
/// case, to a shopping item, which is created the first time, so the item's category
/// carries over. Rebuilding the list from its plan leaves the line alone.
pub async fn add_custom(
    pool: &PgPool,
    shopping_list_id: Uuid,
    name: &str,
) -> Result<ShoppingListItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingListItem,
        r#"
        WITH item AS (
            INSERT INTO shopping_item (name)
            VALUES (btrim($2))
            ON CONFLICT ((lower(name))) DO UPDATE SET name = shopping_item.name
            RETURNING id
        )
        INSERT INTO shopping_list_item (shopping_list_id, shopping_item_id)
        SELECT $1, item.id FROM item
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
        "#,
        shopping_list_id,
        name,
    )
    .fetch_one(pool)
    .await
}

/// Put the ingredient or shopping item behind a line into `category_id`, or take it out
/// of any with `None`. Every list showing it follows.
pub async fn set_category(
    pool: &PgPool,
    id: Uuid,
    category_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        WITH line AS (
            SELECT ingredient_id, shopping_item_id FROM shopping_list_item WHERE id = $1
        ), ingredient_set AS (
            UPDATE ingredient
            SET shopping_category_id = $2
            FROM line
            WHERE ingredient.id = line.ingredient_id
            RETURNING ingredient.id
        ), item_set AS (
            UPDATE shopping_item
            SET shopping_category_id = $2
            FROM line
            WHERE shopping_item.id = line.shopping_item_id
            RETURNING shopping_item.id
        )
        SELECT id AS "id!" FROM ingredient_set
        UNION ALL
        SELECT id FROM item_set
        "#,
        id,
        category_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(())
}

/// Remove a line that is not an ingredient. An ingredient line is not found here: it goes when the
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
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
        "#,
        id,
        shopping_list_id,
    )
    .fetch_one(pool)
    .await
}

/// Clear every ticked line of a list and say how many went. Lines that are not
/// ingredients are deleted; ingredient lines are hidden instead, so a rebuild from the plan does
/// not bring them back.
pub async fn clear_bought(pool: &PgPool, shopping_list_id: Uuid) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let removed = sqlx::query_scalar!(
        r#"
        DELETE FROM shopping_list_item
        WHERE shopping_list_id = $1 AND ingredient_id IS NULL AND is_purchased
        RETURNING id
        "#,
        shopping_list_id,
    )
    .fetch_all(&mut *tx)
    .await?;
    let hidden = sqlx::query_scalar!(
        r#"
        UPDATE shopping_list_item
        SET is_cleared = true
        WHERE shopping_list_id = $1 AND ingredient_id IS NOT NULL AND is_purchased AND NOT is_cleared
        RETURNING id
        "#,
        shopping_list_id,
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((removed.len() + hidden.len()) as u64)
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
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
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
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
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
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, shopping_item_id, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}
