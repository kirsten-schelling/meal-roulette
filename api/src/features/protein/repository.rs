use crate::references::{to_references, ReferenceRow};
use domain::protein::{NewProtein, Protein, ProteinWithRecipeList, UpdateProtein};
use domain::usage::{Reference, ReferenceKind, UsageRef};
use sqlx::PgPool;
use uuid::Uuid;

struct ProteinRow {
    id: Uuid,
    household_id: Option<Uuid>,
    name: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    titles: Vec<String>,
    recipe_ids: Vec<Uuid>,
}

impl From<ProteinRow> for ProteinWithRecipeList {
    fn from(row: ProteinRow) -> Self {
        ProteinWithRecipeList {
            protein: Protein {
                id: row.id,
                household_id: row.household_id,
                name: row.name,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            used_by: row
                .recipe_ids
                .into_iter()
                .zip(row.titles)
                .map(|(id, name)| UsageRef { id, name })
                .collect(),
        }
    }
}

pub async fn list(pool: &PgPool) -> Result<Vec<ProteinWithRecipeList>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ProteinRow,
        r#"
        SELECT protein.id, protein.household_id, protein.name,
               protein.created_at, protein.updated_at,
               COALESCE(array_agg(recipe.title ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "titles!",
               COALESCE(array_agg(recipe.id ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "recipe_ids!"
        FROM protein
        LEFT JOIN recipe ON recipe.main_protein_id = protein.id
        GROUP BY protein.id
        ORDER BY protein.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<ProteinWithRecipeList, sqlx::Error> {
    let row = sqlx::query_as!(
        ProteinRow,
        r#"
        SELECT protein.id, protein.household_id, protein.name,
               protein.created_at, protein.updated_at,
               COALESCE(array_agg(recipe.title ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "titles!",
               COALESCE(array_agg(recipe.id ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "recipe_ids!"
        FROM protein
        LEFT JOIN recipe ON recipe.main_protein_id = protein.id
        WHERE protein.id = $1
        GROUP BY protein.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
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

pub async fn references(pool: &PgPool, id: Uuid) -> Result<Vec<Reference>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ReferenceRow,
        r#"
        SELECT recipe.id, recipe.title AS name
        FROM recipe
        WHERE recipe.main_protein_id = $1
        ORDER BY recipe.title, recipe.id
        "#,
        id,
    )
    .fetch_all(pool)
    .await?;
    Ok(to_references(rows, ReferenceKind::Recipe))
}

/// Detaches the recipes (their protein becomes empty) and deletes the protein in one transaction.
pub async fn delete_detaching(pool: &PgPool, id: Uuid) -> Result<Protein, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query_scalar!(
        "UPDATE recipe SET main_protein_id = NULL WHERE main_protein_id = $1 RETURNING id",
        id,
    )
    .fetch_all(&mut *tx)
    .await?;
    let deleted = sqlx::query_as!(
        Protein,
        r#"
        DELETE FROM protein
        WHERE id = $1
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(deleted)
}
