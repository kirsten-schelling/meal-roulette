use super::model::{Ingredient, NewIngredient, UpdateIngredient};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Ingredient>, sqlx::Error> {
    let ingredients = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT ingredient.id, ingredient.name, ingredient.purchase_size, unit.name as unit, ingredient.unit_id, ingredient.created_at, ingredient.updated_at
        FROM ingredient
        JOIN unit ON ingredient.unit_id = unit.id
        ORDER BY ingredient.created_at DESC
    "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(ingredients)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Ingredient, sqlx::Error> {
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT ingredient.id, ingredient.name, ingredient.purchase_size, unit.name as unit, ingredient.unit_id, ingredient.created_at, ingredient.updated_at
        FROM ingredient
        JOIN unit ON ingredient.unit_id = unit.id
        WHERE ingredient.id = $1
    "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(ingredient)
}

pub async fn create(pool: &PgPool, ingredient: NewIngredient) -> Result<Ingredient, sqlx::Error> {
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with new_ingredient as (
            INSERT INTO ingredient (name, purchase_size, unit_id) VALUES ($1, $2, $3) RETURNING id, name, purchase_size, unit_id, created_at, updated_at
        )
        SELECT new_ingredient.id, new_ingredient.name, new_ingredient.purchase_size, unit.name as unit, new_ingredient.unit_id, new_ingredient.created_at, new_ingredient.updated_at
        FROM new_ingredient
        JOIN unit ON new_ingredient.unit_id = unit.id
        "#,
        ingredient.name,
        ingredient.purchase_size,
        ingredient.unit_id,
        )
        .fetch_one(pool)
        .await?;
    Ok(ingredient)
}
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    ingredient: UpdateIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with updated_ingredient as (
            UPDATE ingredient SET name = COALESCE($1, name), purchase_size = COALESCE($2, purchase_size), unit_id = COALESCE($3, unit_id) WHERE id = $4 RETURNING id, name, purchase_size, unit_id, created_at, updated_at
        )
        SELECT updated_ingredient.id, updated_ingredient.name, updated_ingredient.purchase_size, unit.name as unit, updated_ingredient.unit_id, updated_ingredient.created_at, updated_ingredient.updated_at
        FROM updated_ingredient
        JOIN unit ON updated_ingredient.unit_id = unit.id
        "#,
        ingredient.name,
        ingredient.purchase_size,
        ingredient.unit_id,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(ingredient)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Ingredient, sqlx::Error> {
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with deleted_ingredient as (
            DELETE FROM ingredient WHERE id = $1 RETURNING id, name, purchase_size, unit_id, created_at, updated_at
        )
        SELECT deleted_ingredient.id, deleted_ingredient.name, deleted_ingredient.purchase_size, unit.name as unit, deleted_ingredient.unit_id, deleted_ingredient.created_at, deleted_ingredient.updated_at
        FROM deleted_ingredient
        JOIN unit ON deleted_ingredient.unit_id = unit.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(ingredient)
}
