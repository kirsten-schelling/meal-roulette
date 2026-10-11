use domain::unit::{NewUnit, Unit, UnitEntry, UnitWithIngredientList, UpdateUnit};
use domain::usage::UsageRef;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

struct UnitRow {
    id: Uuid,
    household_id: Option<Uuid>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    name: String,
    description: Option<String>,
    names: Vec<String>,
    ingredient_ids: Vec<Uuid>,
}

impl From<UnitRow> for UnitWithIngredientList {
    fn from(row: UnitRow) -> Self {
        UnitWithIngredientList {
            unit: Unit {
                id: row.id,
                household_id: row.household_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                name: row.name,
                description: row.description,
            },
            used_by: row
                .ingredient_ids
                .into_iter()
                .zip(row.names)
                .map(|(id, name)| UsageRef { id, name })
                .collect(),
        }
    }
}

pub async fn list(pool: &PgPool) -> Result<Vec<UnitWithIngredientList>, sqlx::Error> {
    let rows = sqlx::query_as!(
        UnitRow,
        r#"
        SELECT unit.id, unit.household_id, unit.created_at, unit.updated_at,
               unit.name, unit.description,
               COALESCE(array_agg(ingredient.name ORDER BY ingredient.name, ingredient.id)
                        FILTER (WHERE ingredient.id IS NOT NULL), '{}') AS "names!",
               COALESCE(array_agg(ingredient.id ORDER BY ingredient.name, ingredient.id)
                        FILTER (WHERE ingredient.id IS NOT NULL), '{}') AS "ingredient_ids!"
        FROM unit
        LEFT JOIN ingredient ON ingredient.unit_id = unit.id
        GROUP BY unit.id
        ORDER BY unit.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<UnitWithIngredientList, sqlx::Error> {
    let row = sqlx::query_as!(
        UnitRow,
        r#"
        SELECT unit.id, unit.household_id, unit.created_at, unit.updated_at,
               unit.name, unit.description,
               COALESCE(array_agg(ingredient.name ORDER BY ingredient.name, ingredient.id)
                        FILTER (WHERE ingredient.id IS NOT NULL), '{}') AS "names!",
               COALESCE(array_agg(ingredient.id ORDER BY ingredient.name, ingredient.id)
                        FILTER (WHERE ingredient.id IS NOT NULL), '{}') AS "ingredient_ids!"
        FROM unit
        LEFT JOIN ingredient ON ingredient.unit_id = unit.id
        WHERE unit.id = $1
        GROUP BY unit.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
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

/// A unit typed as new reuses one that already has that name, so two new ingredients
/// measured in the same new unit (or a name differing only in case) share one row.
pub async fn resolve(conn: &mut PgConnection, entry: UnitEntry) -> Result<Uuid, sqlx::Error> {
    let name = match entry {
        UnitEntry::Existing(id) => return Ok(id),
        UnitEntry::New(name) => name.trim().to_string(),
    };
    let existing = sqlx::query_scalar!(
        r#"
        SELECT id FROM unit
        WHERE household_id IS NULL AND lower(name) = lower($1)
        "#,
        name,
    )
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let unit = create(
        conn,
        NewUnit {
            household_id: None,
            name,
            description: None,
        },
    )
    .await?;
    Ok(unit.id)
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
