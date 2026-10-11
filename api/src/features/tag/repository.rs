use domain::tag::{NewTag, Tag, TagEntry, TagWithRecipeList, UpdateTag};
use domain::usage::UsageRef;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

struct TagRow {
    id: Uuid,
    household_id: Option<Uuid>,
    name: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    titles: Vec<String>,
    recipe_ids: Vec<Uuid>,
}

impl From<TagRow> for TagWithRecipeList {
    fn from(row: TagRow) -> Self {
        TagWithRecipeList {
            tag: Tag {
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

pub async fn list(pool: &PgPool) -> Result<Vec<TagWithRecipeList>, sqlx::Error> {
    let rows = sqlx::query_as!(
        TagRow,
        r#"
        SELECT tag.id, tag.household_id, tag.name, tag.created_at, tag.updated_at,
               COALESCE(array_agg(recipe.title ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "titles!",
               COALESCE(array_agg(recipe.id ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "recipe_ids!"
        FROM tag
        LEFT JOIN recipe_tag ON recipe_tag.tag_id = tag.id
        LEFT JOIN recipe ON recipe.id = recipe_tag.recipe_id
        GROUP BY tag.id
        ORDER BY tag.name
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<TagWithRecipeList, sqlx::Error> {
    let row = sqlx::query_as!(
        TagRow,
        r#"
        SELECT tag.id, tag.household_id, tag.name, tag.created_at, tag.updated_at,
               COALESCE(array_agg(recipe.title ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "titles!",
               COALESCE(array_agg(recipe.id ORDER BY recipe.title, recipe.id)
                        FILTER (WHERE recipe.id IS NOT NULL), '{}') AS "recipe_ids!"
        FROM tag
        LEFT JOIN recipe_tag ON recipe_tag.tag_id = tag.id
        LEFT JOIN recipe ON recipe.id = recipe_tag.recipe_id
        WHERE tag.id = $1
        GROUP BY tag.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn create<'e, E>(executor: E, new_tag: NewTag) -> Result<Tag, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let tag = sqlx::query_as!(
        Tag,
        r#"
        INSERT INTO tag (household_id, name)
        VALUES ($1, $2)
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        new_tag.household_id,
        new_tag.name.trim(),
    )
    .fetch_one(executor)
    .await?;
    Ok(tag)
}

/// A tag typed as new reuses one that already has that name (ignoring case).
pub async fn resolve(conn: &mut PgConnection, entry: TagEntry) -> Result<Uuid, sqlx::Error> {
    let name = match entry {
        TagEntry::Existing(id) => return Ok(id),
        TagEntry::New(name) => name.trim().to_string(),
    };
    let existing = sqlx::query_scalar!(
        r#"
        SELECT id FROM tag
        WHERE household_id IS NULL AND lower(name) = lower($1)
        "#,
        name,
    )
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let tag = create(
        conn,
        NewTag {
            household_id: None,
            name,
        },
    )
    .await?;
    Ok(tag.id)
}

pub async fn update(pool: &PgPool, id: Uuid, tag: UpdateTag) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as!(
        Tag,
        r#"
        UPDATE tag
        SET name = COALESCE($1, name)
        WHERE id = $2
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        tag.name.as_deref().map(str::trim),
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(tag)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as!(
        Tag,
        r#"
        DELETE FROM tag
        WHERE id = $1
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(tag)
}
