use domain::protein::{NewProtein, Protein, UpdateProtein};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Protein>, sqlx::Error> {
    let proteins = sqlx::query_as!(
        Protein,
        r#"
        SELECT id, household_id, name, created_at, updated_at
        FROM protein
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(proteins)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Protein, sqlx::Error> {
    let protein = sqlx::query_as!(
        Protein,
        r#"
        SELECT id, household_id, name, created_at, updated_at
        FROM protein
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(protein)
}

pub async fn create<'e, E>(executor: E, new_protein: NewProtein) -> Result<Protein, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let protein = sqlx::query_as!(
        Protein,
        r#"
        INSERT INTO protein (household_id, name)
        VALUES ($1, $2)
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        new_protein.household_id,
        new_protein.name,
    )
    .fetch_one(executor)
    .await?;
    Ok(protein)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    protein: UpdateProtein,
) -> Result<Protein, sqlx::Error> {
    let protein = sqlx::query_as!(
        Protein,
        r#"
        UPDATE protein
        SET name = COALESCE($1, name)
        WHERE id = $2
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        protein.name,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(protein)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Protein, sqlx::Error> {
    let protein = sqlx::query_as!(
        Protein,
        r#"
        DELETE FROM protein
        WHERE id = $1
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(protein)
}
