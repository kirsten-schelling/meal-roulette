use domain::roll_preset::{NewRollPreset, RollPreset, UpdateRollPreset};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<RollPreset>, sqlx::Error> {
    let presets = sqlx::query_as!(
        RollPreset,
        r#"
        SELECT id, household_id, name, settings AS "settings!: serde_json::Value",
               is_default, created_at, updated_at
        FROM roll_preset
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(presets)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<RollPreset, sqlx::Error> {
    let preset = sqlx::query_as!(
        RollPreset,
        r#"
        SELECT id, household_id, name, settings AS "settings!: serde_json::Value",
               is_default, created_at, updated_at
        FROM roll_preset
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(preset)
}

pub async fn create(pool: &PgPool, new_preset: NewRollPreset) -> Result<RollPreset, sqlx::Error> {
    let preset = sqlx::query_as!(
        RollPreset,
        r#"
        INSERT INTO roll_preset (household_id, name, settings)
        VALUES ($1, $2, $3)
        RETURNING id, household_id, name, settings AS "settings!: serde_json::Value",
                  is_default, created_at, updated_at
        "#,
        new_preset.household_id,
        new_preset.name,
        new_preset.settings,
    )
    .fetch_one(pool)
    .await?;
    Ok(preset)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    preset: UpdateRollPreset,
) -> Result<RollPreset, sqlx::Error> {
    let preset = sqlx::query_as!(
        RollPreset,
        r#"
        UPDATE roll_preset
        SET name = COALESCE($1, name),
            settings = COALESCE($2, settings)
        WHERE id = $3
        RETURNING id, household_id, name, settings AS "settings!: serde_json::Value",
                  is_default, created_at, updated_at
        "#,
        preset.name,
        preset.settings,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(preset)
}

/// Make `id` the sole default for its household: clear the current default in
/// that household, then set this one. Wrapped in a transaction so the partial
/// unique index (one default per household) never sees two.
pub async fn set_default(pool: &PgPool, id: Uuid) -> Result<RollPreset, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Clear the existing default (RETURNING keeps this compile-time checked,
    // per the query_as!-only lint; the cleared ids aren't needed).
    let _cleared = sqlx::query_scalar!(
        r#"
        UPDATE roll_preset
        SET is_default = false
        WHERE is_default
          AND id <> $1
          AND household_id IS NOT DISTINCT FROM
              (SELECT household_id FROM roll_preset WHERE id = $1)
        RETURNING id
        "#,
        id,
    )
    .fetch_all(&mut *tx)
    .await?;

    let preset = sqlx::query_as!(
        RollPreset,
        r#"
        UPDATE roll_preset
        SET is_default = true
        WHERE id = $1
        RETURNING id, household_id, name, settings AS "settings!: serde_json::Value",
                  is_default, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(preset)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<RollPreset, sqlx::Error> {
    let preset = sqlx::query_as!(
        RollPreset,
        r#"
        DELETE FROM roll_preset
        WHERE id = $1
        RETURNING id, household_id, name, settings AS "settings!: serde_json::Value",
                  is_default, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(preset)
}
