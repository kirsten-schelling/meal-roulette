use domain::cuisine::{Cuisine, NewCuisine, UpdateCuisine};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Cuisine>, sqlx::Error> {
    let cuisines = sqlx::query_as!(
        Cuisine,
        r#"
        SELECT id, household_id, name, created_at, updated_at
        FROM cuisine
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(cuisines)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Cuisine, sqlx::Error> {
    let cuisine = sqlx::query_as!(
        Cuisine,
        r#"
        SELECT id, household_id, name, created_at, updated_at
        FROM cuisine
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(cuisine)
}

pub async fn create<'e, E>(executor: E, new_cuisine: NewCuisine) -> Result<Cuisine, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let cuisine = sqlx::query_as!(
        Cuisine,
        r#"
        INSERT INTO cuisine (household_id, name)
        VALUES ($1, $2)
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        new_cuisine.household_id,
        new_cuisine.name,
    )
    .fetch_one(executor)
    .await?;
    Ok(cuisine)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    cuisine: UpdateCuisine,
) -> Result<Cuisine, sqlx::Error> {
    let cuisine = sqlx::query_as!(
        Cuisine,
        r#"
        UPDATE cuisine
        SET name = COALESCE($1, name)
        WHERE id = $2
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        cuisine.name,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(cuisine)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Cuisine, sqlx::Error> {
    let cuisine = sqlx::query_as!(
        Cuisine,
        r#"
        DELETE FROM cuisine
        WHERE id = $1
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(cuisine)
}
