use crate::features::unit;
use domain::ingredient::{
    Ingredient, IngredientEntry, InsertIngredient, NewIngredient, UpdateIngredient,
};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Ingredient>, sqlx::Error> {
    let ingredients = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT ingredient.id, ingredient.household_id, ingredient.name, ingredient.purchase_size, unit.name as unit, ingredient.unit_id, ingredient.created_at, ingredient.updated_at
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
        SELECT ingredient.id, ingredient.household_id, ingredient.name, ingredient.purchase_size, unit.name as unit, ingredient.unit_id, ingredient.created_at, ingredient.updated_at
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

pub async fn create<'e, E>(
    executor: E,
    ingredient: InsertIngredient,
) -> Result<Ingredient, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with new_ingredient as (
            INSERT INTO ingredient (household_id, name, purchase_size, unit_id) VALUES ($1, $2, $3, $4) RETURNING id, household_id, name, purchase_size, unit_id, created_at, updated_at
        )
        SELECT new_ingredient.id, new_ingredient.household_id, new_ingredient.name, new_ingredient.purchase_size, unit.name as unit, new_ingredient.unit_id, new_ingredient.created_at, new_ingredient.updated_at
        FROM new_ingredient
        JOIN unit ON new_ingredient.unit_id = unit.id
        "#,
        ingredient.household_id,
        ingredient.name,
        ingredient.purchase_size,
        ingredient.unit_id,
        )
        .fetch_one(executor)
        .await?;
    Ok(ingredient)
}

/// Like units, an ingredient typed as new reuses one that already has that name.
pub async fn resolve(conn: &mut PgConnection, entry: IngredientEntry) -> Result<Uuid, sqlx::Error> {
    let (name, purchase_size, unit) = match entry {
        IngredientEntry::Existing(id) => return Ok(id),
        IngredientEntry::New {
            name,
            purchase_size,
            unit,
        } => (name.trim().to_string(), purchase_size, unit),
    };
    let existing = sqlx::query_scalar!(
        r#"
        SELECT id FROM ingredient
        WHERE household_id IS NULL AND lower(name) = lower($1)
        "#,
        name,
    )
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let unit_id = unit::repository::resolve(&mut *conn, unit).await?;
    let ingredient = create(
        conn,
        InsertIngredient {
            household_id: None,
            name,
            purchase_size,
            unit_id,
        },
    )
    .await?;
    Ok(ingredient.id)
}

pub async fn create_with_unit(
    pool: &PgPool,
    new_ingredient: NewIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let unit_id = unit::repository::resolve(&mut tx, new_ingredient.unit).await?;
    let ingredient = create(
        &mut *tx,
        InsertIngredient {
            household_id: new_ingredient.household_id,
            name: new_ingredient.name,
            purchase_size: new_ingredient.purchase_size,
            unit_id,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(ingredient)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    ingredient: UpdateIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let unit_id = unit::repository::resolve(&mut tx, ingredient.unit).await?;
    let updated = sqlx::query_as!(
        Ingredient,
        r#"
        with updated_ingredient as (
            UPDATE ingredient SET name = $1, purchase_size = $2, unit_id = $3 WHERE id = $4 RETURNING id, household_id, name, purchase_size, unit_id, created_at, updated_at
        )
        SELECT updated_ingredient.id, updated_ingredient.household_id, updated_ingredient.name, updated_ingredient.purchase_size, unit.name as unit, updated_ingredient.unit_id, updated_ingredient.created_at, updated_ingredient.updated_at
        FROM updated_ingredient
        JOIN unit ON updated_ingredient.unit_id = unit.id
        "#,
        ingredient.name.trim(),
        ingredient.purchase_size,
        unit_id,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Ingredient, sqlx::Error> {
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with deleted_ingredient as (
            DELETE FROM ingredient WHERE id = $1 RETURNING id, household_id, name, purchase_size, unit_id, created_at, updated_at
        )
        SELECT deleted_ingredient.id, deleted_ingredient.household_id, deleted_ingredient.name, deleted_ingredient.purchase_size, unit.name as unit, deleted_ingredient.unit_id, deleted_ingredient.created_at, deleted_ingredient.updated_at
        FROM deleted_ingredient
        JOIN unit ON deleted_ingredient.unit_id = unit.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(ingredient)
}
