use domain::shopping_list_item::{NewShoppingListItem, ShoppingListItem, UpdateShoppingListItem};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<ShoppingListItem>, sqlx::Error> {
    let items = sqlx::query_as!(
        ShoppingListItem,
        r#"
        SELECT id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, created_at, updated_at
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
        SELECT id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, created_at, updated_at
        FROM shopping_list_item
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
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
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, created_at, updated_at
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
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, created_at, updated_at
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
        RETURNING id, shopping_list_id, ingredient_id, unit_id, qty, is_purchased, note, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}
