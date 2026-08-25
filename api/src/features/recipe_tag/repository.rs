use domain::recipe_tag::{NewRecipeTag, RecipeTag};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<RecipeTag>, sqlx::Error> {
    let links = sqlx::query_as!(
        RecipeTag,
        r#"
        SELECT id, recipe_id, tag_id, created_at, updated_at
        FROM recipe_tag
        ORDER BY created_at
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(links)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<RecipeTag, sqlx::Error> {
    let link = sqlx::query_as!(
        RecipeTag,
        r#"
        SELECT id, recipe_id, tag_id, created_at, updated_at
        FROM recipe_tag
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(link)
}

pub async fn create(pool: &PgPool, new_link: NewRecipeTag) -> Result<RecipeTag, sqlx::Error> {
    let link = sqlx::query_as!(
        RecipeTag,
        r#"
        INSERT INTO recipe_tag (recipe_id, tag_id)
        VALUES ($1, $2)
        RETURNING id, recipe_id, tag_id, created_at, updated_at
        "#,
        new_link.recipe_id,
        new_link.tag_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(link)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<RecipeTag, sqlx::Error> {
    let link = sqlx::query_as!(
        RecipeTag,
        r#"
        DELETE FROM recipe_tag
        WHERE id = $1
        RETURNING id, recipe_id, tag_id, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(link)
}
