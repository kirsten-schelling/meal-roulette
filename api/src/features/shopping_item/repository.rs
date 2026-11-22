use domain::shopping_item::ShoppingItem;
use sqlx::PgPool;
use uuid::Uuid;

/// Every item, by name.
pub async fn list(pool: &PgPool) -> Result<Vec<ShoppingItem>, sqlx::Error> {
    sqlx::query_as!(
        ShoppingItem,
        r#"
        SELECT id, name, shopping_category_id
        FROM shopping_item
        ORDER BY lower(name)
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    shopping_category_id: Option<Uuid>,
) -> Result<ShoppingItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingItem,
        r#"
        INSERT INTO shopping_item (name, shopping_category_id)
        VALUES (btrim($1), $2)
        RETURNING id, name, shopping_category_id
        "#,
        name,
        shopping_category_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    shopping_category_id: Option<Uuid>,
) -> Result<ShoppingItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingItem,
        r#"
        UPDATE shopping_item
        SET name = btrim($2), shopping_category_id = $3
        WHERE id = $1
        RETURNING id, name, shopping_category_id
        "#,
        id,
        name,
        shopping_category_id,
    )
    .fetch_one(pool)
    .await
}

/// Delete an item. Any shopping list lines for it go with it.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<ShoppingItem, sqlx::Error> {
    sqlx::query_as!(
        ShoppingItem,
        r#"
        DELETE FROM shopping_item
        WHERE id = $1
        RETURNING id, name, shopping_category_id
        "#,
        id,
    )
    .fetch_one(pool)
    .await
}
