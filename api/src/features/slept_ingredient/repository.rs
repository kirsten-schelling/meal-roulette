use crate::error::AppError;
use domain::slept_ingredient::{
    NewSleptIngredient, SleptIngredient, UpdateSleptIngredient, MAX_SLEEP_WEEKS,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<SleptIngredient>, sqlx::Error> {
    let slept = sqlx::query_as!(
        SleptIngredient,
        r#"
        SELECT id, household_id, ingredient_id, sleep_until, created_at, updated_at
        FROM slept_ingredient
        ORDER BY sleep_until
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(slept)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<SleptIngredient, sqlx::Error> {
    let slept = sqlx::query_as!(
        SleptIngredient,
        r#"
        SELECT id, household_id, ingredient_id, sleep_until, created_at, updated_at
        FROM slept_ingredient
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(slept)
}

pub async fn create(
    pool: &PgPool,
    new_slept: NewSleptIngredient,
) -> Result<SleptIngredient, sqlx::Error> {
    let slept = sqlx::query_as!(
        SleptIngredient,
        r#"
        INSERT INTO slept_ingredient (household_id, ingredient_id, sleep_until)
        VALUES ($1, $2, $3)
        RETURNING id, household_id, ingredient_id, sleep_until, created_at, updated_at
        "#,
        new_slept.household_id,
        new_slept.ingredient_id,
        new_slept.sleep_until,
    )
    .fetch_one(pool)
    .await?;
    Ok(slept)
}

/// Sleep each ingredient for `weeks` weeks from today, replacing any sleep it
/// already has.
pub async fn sleep_for_weeks(
    pool: &PgPool,
    household_id: Uuid,
    ingredient_ids: &[Uuid],
    weeks: i32,
) -> Result<Vec<SleptIngredient>, AppError> {
    if !(1..=MAX_SLEEP_WEEKS).contains(&weeks) {
        return Err(AppError::BadRequest(format!(
            "Sleep for between 1 and {MAX_SLEEP_WEEKS} weeks."
        )));
    }
    let slept = sqlx::query_as!(
        SleptIngredient,
        r#"
        INSERT INTO slept_ingredient (household_id, ingredient_id, sleep_until)
        SELECT DISTINCT $1::uuid, picked.ingredient_id, CURRENT_DATE + 7 * $3::int
        FROM UNNEST($2::uuid[]) AS picked(ingredient_id)
        ON CONFLICT ON CONSTRAINT uq_slept_ingredient
        DO UPDATE SET sleep_until = EXCLUDED.sleep_until
        RETURNING id, household_id, ingredient_id, sleep_until, created_at, updated_at
        "#,
        household_id,
        ingredient_ids,
        weeks,
    )
    .fetch_all(pool)
    .await?;
    Ok(slept)
}

/// End an ingredient's sleep so it goes back on the list. `None` when it was
/// not asleep.
pub async fn wake(
    pool: &PgPool,
    household_id: Uuid,
    ingredient_id: Uuid,
) -> Result<Option<SleptIngredient>, sqlx::Error> {
    sqlx::query_as!(
        SleptIngredient,
        r#"
        DELETE FROM slept_ingredient
        WHERE household_id = $1 AND ingredient_id = $2
        RETURNING id, household_id, ingredient_id, sleep_until, created_at, updated_at
        "#,
        household_id,
        ingredient_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    slept: UpdateSleptIngredient,
) -> Result<SleptIngredient, sqlx::Error> {
    let slept = sqlx::query_as!(
        SleptIngredient,
        r#"
        UPDATE slept_ingredient
        SET ingredient_id = COALESCE($1, ingredient_id),
            sleep_until = COALESCE($2, sleep_until)
        WHERE id = $3
        RETURNING id, household_id, ingredient_id, sleep_until, created_at, updated_at
        "#,
        slept.ingredient_id,
        slept.sleep_until,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(slept)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<SleptIngredient, sqlx::Error> {
    let slept = sqlx::query_as!(
        SleptIngredient,
        r#"
        DELETE FROM slept_ingredient
        WHERE id = $1
        RETURNING id, household_id, ingredient_id, sleep_until, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(slept)
}
