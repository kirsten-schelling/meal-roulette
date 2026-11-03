use crate::references::{to_references, ReferenceRow};
use domain::cuisine::{Cuisine, CuisineWithRecipeList, NewCuisine, UpdateCuisine};
use domain::usage::{Reference, ReferenceKind, UsageRef};
use sqlx::PgPool;
use uuid::Uuid;

struct CuisineRow {
    id: Uuid,
    household_id: Option<Uuid>,
    name: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    titles: Vec<String>,
    recipe_ids: Vec<Uuid>,
}

impl From<CuisineRow> for CuisineWithRecipeList {
    fn from(row: CuisineRow) -> Self {
        CuisineWithRecipeList {
            cuisine: Cuisine {
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

pub async fn list(pool: &PgPool) -> Result<Vec<CuisineWithRecipeList>, sqlx::Error> {
    let rows = sqlx::query_as!(
        CuisineRow,
        r#"
        SELECT cuisine.id, cuisine.household_id, cuisine.name,
               cuisine.created_at, cuisine.updated_at,
               COALESCE(array_agg(recipe.title ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "titles!",
               COALESCE(array_agg(recipe.id ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "recipe_ids!"
        FROM cuisine
        LEFT JOIN recipe ON recipe.cuisine_id = cuisine.id
        GROUP BY cuisine.id
        ORDER BY cuisine.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<CuisineWithRecipeList, sqlx::Error> {
    let row = sqlx::query_as!(
        CuisineRow,
        r#"
        SELECT cuisine.id, cuisine.household_id, cuisine.name,
               cuisine.created_at, cuisine.updated_at,
               COALESCE(array_agg(recipe.title ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "titles!",
               COALESCE(array_agg(recipe.id ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "recipe_ids!"
        FROM cuisine
        LEFT JOIN recipe ON recipe.cuisine_id = cuisine.id
        WHERE cuisine.id = $1
        GROUP BY cuisine.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
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

pub async fn references(pool: &PgPool, id: Uuid) -> Result<Vec<Reference>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ReferenceRow,
        r#"
        SELECT recipe.id, recipe.title AS name
        FROM recipe
        WHERE recipe.cuisine_id = $1
        ORDER BY recipe.title, recipe.id
        "#,
        id,
    )
    .fetch_all(pool)
    .await?;
    Ok(to_references(rows, ReferenceKind::Recipe))
}

/// Detaches the recipes (their cuisine becomes empty) and deletes the cuisine in one transaction.
pub async fn delete_detaching(pool: &PgPool, id: Uuid) -> Result<Cuisine, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query_scalar!(
        "UPDATE recipe SET cuisine_id = NULL WHERE cuisine_id = $1 RETURNING id",
        id,
    )
    .fetch_all(&mut *tx)
    .await?;
    let deleted = sqlx::query_as!(
        Cuisine,
        r#"
        DELETE FROM cuisine
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
