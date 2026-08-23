use super::model::{NewRecipe, Recipe, UpdateRecipe};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Recipe>, sqlx::Error> {
    let recipes = sqlx::query_as!(
        Recipe,
        r#"
        SELECT id, title, difficulty, time_mins, created_at, updated_at
        FROM recipe
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(recipes)
}

pub async fn create(pool: &PgPool, new_recipe: NewRecipe) -> Result<Recipe, sqlx::Error> {
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        INSERT INTO recipe (title, difficulty, time_mins)
        VALUES ($1, $2, $3)
        RETURNING id, title, difficulty, time_mins, created_at, updated_at
        "#,
        new_recipe.title,
        new_recipe.difficulty,
        new_recipe.time_mins,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Recipe, sqlx::Error> {
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        SELECT id, title, difficulty, time_mins, created_at, updated_at
        FROM recipe
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    update_recipe: UpdateRecipe,
) -> Result<Recipe, sqlx::Error> {
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        UPDATE recipe
        SET title = COALESCE($1, title),
            difficulty = COALESCE($2, difficulty),
            time_mins = COALESCE($3, time_mins)
        WHERE id = $4
        RETURNING id, title, difficulty, time_mins, created_at, updated_at
        "#,
        update_recipe.title,
        update_recipe.difficulty,
        update_recipe.time_mins,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Recipe, sqlx::Error> {
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        DELETE FROM recipe WHERE id = $1 RETURNING id, title, difficulty, time_mins, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}
