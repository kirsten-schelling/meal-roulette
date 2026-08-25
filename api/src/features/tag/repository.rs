use domain::tag::{NewTag, Tag, UpdateTag};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Tag>, sqlx::Error> {
    let tags = sqlx::query_as!(
        Tag,
        r#"
        SELECT id, household_id, name, created_at, updated_at
        FROM tag
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(tags)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as!(
        Tag,
        r#"
        SELECT id, household_id, name, created_at, updated_at
        FROM tag
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(tag)
}

pub async fn create(pool: &PgPool, new_tag: NewTag) -> Result<Tag, sqlx::Error> {
    let tag = sqlx::query_as!(
        Tag,
        r#"
        INSERT INTO tag (household_id, name)
        VALUES ($1, $2)
        RETURNING id, household_id, name, created_at, updated_at
        "#,
        new_tag.household_id,
        new_tag.name,
    )
    .fetch_one(pool)
    .await?;
    Ok(tag)
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
        tag.name,
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
