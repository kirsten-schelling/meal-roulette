use domain::cook_log::{CookLog, NewCookLog, UpdateCookLog};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<CookLog>, sqlx::Error> {
    let logs = sqlx::query_as!(
        CookLog,
        r#"
        SELECT id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        FROM cook_log
        ORDER BY cooked_on DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(logs)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        SELECT id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        FROM cook_log
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}

pub async fn create(pool: &PgPool, new_log: NewCookLog) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        INSERT INTO cook_log (household_id, recipe_id, planned_meal_id, cooked_on)
        VALUES ($1, $2, $3, $4)
        RETURNING id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        "#,
        new_log.household_id,
        new_log.recipe_id,
        new_log.planned_meal_id,
        new_log.cooked_on,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}

pub async fn update(pool: &PgPool, id: Uuid, log: UpdateCookLog) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        UPDATE cook_log
        SET recipe_id = COALESCE($1, recipe_id),
            planned_meal_id = COALESCE($2, planned_meal_id),
            cooked_on = COALESCE($3, cooked_on)
        WHERE id = $4
        RETURNING id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        "#,
        log.recipe_id,
        log.planned_meal_id,
        log.cooked_on,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        DELETE FROM cook_log
        WHERE id = $1
        RETURNING id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}
