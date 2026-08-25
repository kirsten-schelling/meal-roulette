use domain::household::{Household, NewHousehold, UpdateHousehold};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Household>, sqlx::Error> {
    let households = sqlx::query_as!(
        Household,
        r#"
        SELECT id, name, created_at, updated_at
        FROM household
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(households)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Household, sqlx::Error> {
    let household = sqlx::query_as!(
        Household,
        r#"
        SELECT id, name, created_at, updated_at
        FROM household
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(household)
}

pub async fn create(pool: &PgPool, household: NewHousehold) -> Result<Household, sqlx::Error> {
    let household = sqlx::query_as!(
        Household,
        r#"
        INSERT INTO household (name)
        VALUES ($1)
        RETURNING id, name, created_at, updated_at
        "#,
        household.name,
    )
    .fetch_one(pool)
    .await?;
    Ok(household)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    household: UpdateHousehold,
) -> Result<Household, sqlx::Error> {
    let household = sqlx::query_as!(
        Household,
        r#"
        UPDATE household
        SET name = COALESCE($1, name)
        WHERE id = $2
        RETURNING id, name, created_at, updated_at
        "#,
        household.name,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(household)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Household, sqlx::Error> {
    let household = sqlx::query_as!(
        Household,
        r#"
        DELETE FROM household
        WHERE id = $1
        RETURNING id, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(household)
}
