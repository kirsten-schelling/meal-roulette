// use super::model::{NewRecipe, Recipe, UpdateRecipe};
use domain::recipe::{NewRecipe, Recipe, UpdateRecipe};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Recipe>, sqlx::Error> {
    let recipes = sqlx::query_as!(
        Recipe,
        r#"
        SELECT id, household_id, title, difficulty, time_mins, servings,
               healthiness, main_protein_id, cuisine_id, created_at, updated_at
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
        INSERT INTO recipe
            (household_id, title, difficulty, time_mins, servings,
             healthiness, main_protein_id, cuisine_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, main_protein_id, cuisine_id, created_at, updated_at
        "#,
        new_recipe.household_id,
        new_recipe.title,
        new_recipe.difficulty,
        new_recipe.time_mins,
        new_recipe.servings,
        new_recipe.healthiness,
        new_recipe.main_protein_id,
        new_recipe.cuisine_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Recipe, sqlx::Error> {
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        SELECT id, household_id, title, difficulty, time_mins, servings,
               healthiness, main_protein_id, cuisine_id, created_at, updated_at
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
            time_mins = COALESCE($3, time_mins),
            servings = COALESCE($4, servings),
            healthiness = COALESCE($5, healthiness),
            main_protein_id = COALESCE($6, main_protein_id),
            cuisine_id = COALESCE($7, cuisine_id)
        WHERE id = $8
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, main_protein_id, cuisine_id, created_at, updated_at
        "#,
        update_recipe.title,
        update_recipe.difficulty,
        update_recipe.time_mins,
        update_recipe.servings,
        update_recipe.healthiness,
        update_recipe.main_protein_id,
        update_recipe.cuisine_id,
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
        DELETE FROM recipe
        WHERE id = $1
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, main_protein_id, cuisine_id, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}
