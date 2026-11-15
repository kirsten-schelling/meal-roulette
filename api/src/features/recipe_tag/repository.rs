use domain::recipe_tag::{NewRecipeTag, RecipeTag};
use sqlx::{PgConnection, PgPool};
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

/// Which recipes carry any of `tag_ids`.
pub async fn links_for_tags(
    pool: &PgPool,
    tag_ids: &[Uuid],
) -> Result<Vec<RecipeTag>, sqlx::Error> {
    sqlx::query_as!(
        RecipeTag,
        r#"
        SELECT id, recipe_id, tag_id, created_at, updated_at
        FROM recipe_tag
        WHERE tag_id = ANY($1)
        "#,
        tag_ids,
    )
    .fetch_all(pool)
    .await
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

/// Gives `to_recipe` the same tags as `from_recipe`.
pub async fn copy_tags<'e, E>(
    executor: E,
    from_recipe: Uuid,
    to_recipe: Uuid,
) -> Result<Vec<RecipeTag>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_as!(
        RecipeTag,
        r#"
        INSERT INTO recipe_tag (recipe_id, tag_id)
        SELECT $2, tag_id FROM recipe_tag WHERE recipe_id = $1
        RETURNING id, recipe_id, tag_id, created_at, updated_at
        "#,
        from_recipe,
        to_recipe,
    )
    .fetch_all(executor)
    .await
}

/// Makes `tag_ids` the recipe's whole tag set: links not listed are removed, missing ones
/// added. Returns the links that were added.
pub async fn replace_for_recipe(
    conn: &mut PgConnection,
    recipe_id: Uuid,
    tag_ids: &[Uuid],
) -> Result<Vec<RecipeTag>, sqlx::Error> {
    sqlx::query_as!(
        RecipeTag,
        r#"
        DELETE FROM recipe_tag
        WHERE recipe_id = $1 AND tag_id <> ALL($2)
        RETURNING id, recipe_id, tag_id, created_at, updated_at
        "#,
        recipe_id,
        tag_ids,
    )
    .fetch_all(&mut *conn)
    .await?;
    sqlx::query_as!(
        RecipeTag,
        r#"
        INSERT INTO recipe_tag (recipe_id, tag_id)
        SELECT $1, tag_id FROM unnest($2::uuid[]) AS tag_id
        ON CONFLICT ON CONSTRAINT uq_recipe_tag DO NOTHING
        RETURNING id, recipe_id, tag_id, created_at, updated_at
        "#,
        recipe_id,
        tag_ids,
    )
    .fetch_all(conn)
    .await
}
