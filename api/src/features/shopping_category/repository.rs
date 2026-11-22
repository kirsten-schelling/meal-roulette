use domain::shopping_category::ShoppingCategory;
use sqlx::PgPool;
use uuid::Uuid;

/// Every category in list order.
pub async fn list(pool: &PgPool) -> Result<Vec<ShoppingCategory>, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCategory,
        r#"
        SELECT id, name, position
        FROM shopping_category
        ORDER BY position, name
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Add a category at the end of the list.
pub async fn create(pool: &PgPool, name: &str) -> Result<ShoppingCategory, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCategory,
        r#"
        INSERT INTO shopping_category (name, position)
        SELECT btrim($1), COALESCE(MAX(position), 0) + 1
        FROM shopping_category
        RETURNING id, name, position
        "#,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn rename(pool: &PgPool, id: Uuid, name: &str) -> Result<ShoppingCategory, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCategory,
        r#"
        UPDATE shopping_category
        SET name = btrim($2)
        WHERE id = $1
        RETURNING id, name, position
        "#,
        id,
        name,
    )
    .fetch_one(pool)
    .await
}

/// Put the categories in the order `ids` gives them.
pub async fn reorder(pool: &PgPool, ids: &[Uuid]) -> Result<Vec<ShoppingCategory>, sqlx::Error> {
    let _moved = sqlx::query_scalar!(
        r#"
        UPDATE shopping_category
        SET position = ordered.position::int
        FROM unnest($1::uuid[]) WITH ORDINALITY AS ordered(id, position)
        WHERE shopping_category.id = ordered.id
        RETURNING shopping_category.id
        "#,
        ids,
    )
    .fetch_all(pool)
    .await?;
    list(pool).await
}

/// Delete a category. What was in it becomes uncategorised.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<ShoppingCategory, sqlx::Error> {
    sqlx::query_as!(
        ShoppingCategory,
        r#"
        DELETE FROM shopping_category
        WHERE id = $1
        RETURNING id, name, position
        "#,
        id,
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn categories_are_added_at_the_end_and_can_be_reordered(pool: PgPool) {
        let fruit = create(&pool, " Fruit ").await.unwrap();
        let dairy = create(&pool, "Dairy").await.unwrap();
        assert_eq!(fruit.name, "Fruit");
        assert!(fruit.position < dairy.position);

        let reordered = reorder(&pool, &[dairy.id, fruit.id]).await.unwrap();

        let names: Vec<&str> = reordered.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Dairy", "Fruit"]);
    }
}
