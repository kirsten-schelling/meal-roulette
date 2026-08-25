use domain::planned_meal::{NewPlannedMeal, PlannedMeal, UpdatePlannedMeal};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<PlannedMeal>, sqlx::Error> {
    let meals = sqlx::query_as!(
        PlannedMeal,
        r#"
        SELECT id, meal_plan_id, recipe_id, scheduled_date, is_pinned, target_difficulty, created_at, updated_at
        FROM planned_meal
        ORDER BY scheduled_date
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(meals)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<PlannedMeal, sqlx::Error> {
    let meal = sqlx::query_as!(
        PlannedMeal,
        r#"
        SELECT id, meal_plan_id, recipe_id, scheduled_date, is_pinned, target_difficulty, created_at, updated_at
        FROM planned_meal
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(meal)
}

pub async fn create(pool: &PgPool, new_meal: NewPlannedMeal) -> Result<PlannedMeal, sqlx::Error> {
    let meal = sqlx::query_as!(
        PlannedMeal,
        r#"
        INSERT INTO planned_meal (meal_plan_id, recipe_id, scheduled_date, is_pinned, target_difficulty)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, meal_plan_id, recipe_id, scheduled_date, is_pinned, target_difficulty, created_at, updated_at
        "#,
        new_meal.meal_plan_id,
        new_meal.recipe_id,
        new_meal.scheduled_date,
        new_meal.is_pinned,
        new_meal.target_difficulty,
    )
    .fetch_one(pool)
    .await?;
    Ok(meal)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    meal: UpdatePlannedMeal,
) -> Result<PlannedMeal, sqlx::Error> {
    let meal = sqlx::query_as!(
        PlannedMeal,
        r#"
        UPDATE planned_meal
        SET recipe_id = COALESCE($1, recipe_id),
            scheduled_date = COALESCE($2, scheduled_date),
            is_pinned = COALESCE($3, is_pinned),
            target_difficulty = COALESCE($4, target_difficulty)
        WHERE id = $5
        RETURNING id, meal_plan_id, recipe_id, scheduled_date, is_pinned, target_difficulty, created_at, updated_at
        "#,
        meal.recipe_id,
        meal.scheduled_date,
        meal.is_pinned,
        meal.target_difficulty,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(meal)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<PlannedMeal, sqlx::Error> {
    let meal = sqlx::query_as!(
        PlannedMeal,
        r#"
        DELETE FROM planned_meal
        WHERE id = $1
        RETURNING id, meal_plan_id, recipe_id, scheduled_date, is_pinned, target_difficulty, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(meal)
}
