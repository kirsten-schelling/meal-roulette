use domain::slept_ingredient::{NewSleptIngredient, SleptIngredient, UpdateSleptIngredient};
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
