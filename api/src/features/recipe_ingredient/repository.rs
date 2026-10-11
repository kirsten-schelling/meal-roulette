use crate::features::ingredient;
use domain::recipe_ingredient::{
    NewRecipeIngredient, RecipeIngredient, RecipeIngredientEntry, UpdateRecipeIngredient,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_for_recipe(
    pool: &PgPool,
    recipe_id: Uuid,
) -> Result<Vec<RecipeIngredient>, sqlx::Error> {
    let items = sqlx::query_as!(
        RecipeIngredient,
        r#"
        SELECT
            recipe_ingredient.id,
            recipe_ingredient.created_at,
            recipe_ingredient.updated_at,
            recipe_ingredient.recipe_id,
            recipe_ingredient.ingredient_id,
            ingredient.name AS ingredient_name,
            unit.name AS unit,
            recipe_ingredient.qty,
            recipe_ingredient.note,
            recipe_ingredient.is_optional
        FROM recipe_ingredient
        JOIN ingredient ON recipe_ingredient.ingredient_id = ingredient.id
        JOIN unit ON ingredient.unit_id = unit.id
        WHERE recipe_ingredient.recipe_id = $1
        ORDER BY recipe_ingredient.created_at
        "#,
        recipe_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(items)
}

pub async fn get(
    pool: &PgPool,
    recipe_id: Uuid,
    ingredient_id: Uuid,
) -> Result<RecipeIngredient, sqlx::Error> {
    let item = sqlx::query_as!(
        RecipeIngredient,
        r#"
        SELECT
            recipe_ingredient.id,
            recipe_ingredient.created_at,
            recipe_ingredient.updated_at,
            recipe_ingredient.recipe_id,
            recipe_ingredient.ingredient_id,
            ingredient.name AS ingredient_name,
            unit.name AS unit,
            recipe_ingredient.qty,
            recipe_ingredient.note,
            recipe_ingredient.is_optional
        FROM recipe_ingredient
        JOIN ingredient ON recipe_ingredient.ingredient_id = ingredient.id
        JOIN unit ON ingredient.unit_id = unit.id
        WHERE recipe_ingredient.recipe_id = $1 AND recipe_ingredient.ingredient_id = $2
        "#,
        recipe_id,
        ingredient_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

pub async fn create<'e, E>(
    executor: E,
    recipe_id: Uuid,
    new: NewRecipeIngredient,
) -> Result<RecipeIngredient, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let item = sqlx::query_as!(
        RecipeIngredient,
        r#"
        WITH inserted AS (
            INSERT INTO recipe_ingredient (recipe_id, ingredient_id, qty, note, is_optional)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, created_at, updated_at, recipe_id, ingredient_id, qty, note, is_optional
        )
        SELECT
            inserted.id,
            inserted.created_at,
            inserted.updated_at,
            inserted.recipe_id,
            inserted.ingredient_id,
            ingredient.name AS ingredient_name,
            unit.name AS unit,
            inserted.qty,
            inserted.note,
            inserted.is_optional
        FROM inserted
        JOIN ingredient ON inserted.ingredient_id = ingredient.id
        JOIN unit ON ingredient.unit_id = unit.id
        "#,
        recipe_id,
        new.ingredient_id,
        new.qty,
        new.note,
        new.is_optional,
    )
    .fetch_one(executor)
    .await?;
    Ok(item)
}

pub async fn update(
    pool: &PgPool,
    recipe_id: Uuid,
    ingredient_id: Uuid,
    update: UpdateRecipeIngredient,
) -> Result<RecipeIngredient, sqlx::Error> {
    let item = sqlx::query_as!(
        RecipeIngredient,
        r#"
        WITH updated AS (
            UPDATE recipe_ingredient
            SET qty = $3,
                note = $4,
                is_optional = $5
            WHERE recipe_id = $1 AND ingredient_id = $2
            RETURNING id, created_at, updated_at, recipe_id, ingredient_id, qty, note, is_optional
        )
        SELECT
            updated.id,
            updated.created_at,
            updated.updated_at,
            updated.recipe_id,
            updated.ingredient_id,
            ingredient.name AS ingredient_name,
            unit.name AS unit,
            updated.qty,
            updated.note,
            updated.is_optional
        FROM updated
        JOIN ingredient ON updated.ingredient_id = ingredient.id
        JOIN unit ON ingredient.unit_id = unit.id
        "#,
        recipe_id,
        ingredient_id,
        update.qty,
        update.note,
        update.is_optional,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

/// Adds a line to an existing recipe, creating its ingredient first when it is a new one.
pub async fn add(
    pool: &PgPool,
    recipe_id: Uuid,
    entry: RecipeIngredientEntry,
) -> Result<RecipeIngredient, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let ingredient_id = ingredient::repository::resolve(&mut tx, entry.ingredient).await?;
    let line = create(
        &mut *tx,
        recipe_id,
        NewRecipeIngredient {
            ingredient_id,
            qty: entry.qty,
            note: entry.note,
            is_optional: entry.is_optional,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(line)
}

/// Gives `to_recipe` the same ingredient lines as `from_recipe`.
pub async fn copy_lines<'e, E>(
    executor: E,
    from_recipe: Uuid,
    to_recipe: Uuid,
) -> Result<Vec<RecipeIngredient>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_as!(
        RecipeIngredient,
        r#"
        WITH inserted AS (
            INSERT INTO recipe_ingredient (recipe_id, ingredient_id, qty, note, is_optional)
            SELECT $2, ingredient_id, qty, note, is_optional
            FROM recipe_ingredient
            WHERE recipe_id = $1
            RETURNING id, created_at, updated_at, recipe_id, ingredient_id, qty, note, is_optional
        )
        SELECT
            inserted.id,
            inserted.created_at,
            inserted.updated_at,
            inserted.recipe_id,
            inserted.ingredient_id,
            ingredient.name AS ingredient_name,
            unit.name AS unit,
            inserted.qty,
            inserted.note,
            inserted.is_optional
        FROM inserted
        JOIN ingredient ON inserted.ingredient_id = ingredient.id
        JOIN unit ON ingredient.unit_id = unit.id
        "#,
        from_recipe,
        to_recipe,
    )
    .fetch_all(executor)
    .await
}

pub async fn delete(
    pool: &PgPool,
    recipe_id: Uuid,
    ingredient_id: Uuid,
) -> Result<RecipeIngredient, sqlx::Error> {
    let item = sqlx::query_as!(
        RecipeIngredient,
        r#"
        WITH deleted AS (
            DELETE FROM recipe_ingredient
            WHERE recipe_id = $1 AND ingredient_id = $2
            RETURNING id, created_at, updated_at, recipe_id, ingredient_id, qty, note, is_optional
        )
        SELECT
            deleted.id,
            deleted.created_at,
            deleted.updated_at,
            deleted.recipe_id,
            deleted.ingredient_id,
            ingredient.name AS ingredient_name,
            unit.name AS unit,
            deleted.qty,
            deleted.note,
            deleted.is_optional
        FROM deleted
        JOIN ingredient ON deleted.ingredient_id = ingredient.id
        JOIN unit ON ingredient.unit_id = unit.id
        "#,
        recipe_id,
        ingredient_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

/// One recipe using one ingredient.
pub struct IngredientUse {
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
}

/// Which recipes use any of `ingredient_ids`.
pub async fn uses_of(
    pool: &PgPool,
    ingredient_ids: &[Uuid],
) -> Result<Vec<IngredientUse>, sqlx::Error> {
    sqlx::query_as!(
        IngredientUse,
        r#"
        SELECT recipe_id, ingredient_id
        FROM recipe_ingredient
        WHERE ingredient_id = ANY($1)
        "#,
        ingredient_ids,
    )
    .fetch_all(pool)
    .await
}
