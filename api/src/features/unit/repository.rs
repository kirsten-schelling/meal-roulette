use domain::unit::{NewUnit, Unit, UpdateUnit};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Unit>, sqlx::Error> {
    let units = sqlx::query_as!(
        Unit,
        r#"
        SELECT id, household_id, created_at, updated_at, name, description
        FROM unit
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(units)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Unit, sqlx::Error> {
    let unit = sqlx::query_as!(
        Unit,
        r#"
        SELECT id, household_id, created_at, updated_at, name, description
        FROM unit
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(unit)
}

pub async fn create<'e, E>(executor: E, new_unit: NewUnit) -> Result<Unit, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let unit = sqlx::query_as!(
        Unit,
        r#"
        INSERT INTO unit (household_id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id, household_id, created_at, updated_at, name, description
        "#,
        new_unit.household_id,
        new_unit.name,
        new_unit.description,
    )
    .fetch_one(executor)
    .await?;
    Ok(unit)
}

pub async fn update(pool: &PgPool, id: Uuid, unit: UpdateUnit) -> Result<Unit, sqlx::Error> {
    let unit = sqlx::query_as!(
        Unit,
        r#"
        UPDATE unit
        SET name = COALESCE($1, name),
            description = COALESCE($2, description)
        WHERE id = $3
        RETURNING id, household_id, created_at, updated_at, name, description
        "#,
        unit.name,
        unit.description,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(unit)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Unit, sqlx::Error> {
    let unit = sqlx::query_as!(
        Unit,
        r#"
        DELETE FROM unit
        WHERE id = $1
        RETURNING id, household_id, created_at, updated_at, name, description
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(unit)
}
