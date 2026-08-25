use domain::meal_plan::{MealPlan, NewMealPlan, UpdateMealPlan};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<MealPlan>, sqlx::Error> {
    let plans = sqlx::query_as!(
        MealPlan,
        r#"
        SELECT id, household_id, start_date, seed, name, created_at, updated_at
        FROM meal_plan
        ORDER BY start_date DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(plans)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<MealPlan, sqlx::Error> {
    let plan = sqlx::query_as!(
        MealPlan,
        r#"
        SELECT id, household_id, start_date, seed, name, created_at, updated_at
        FROM meal_plan
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(plan)
}

pub async fn create(pool: &PgPool, new_plan: NewMealPlan) -> Result<MealPlan, sqlx::Error> {
    let plan = sqlx::query_as!(
        MealPlan,
        r#"
        INSERT INTO meal_plan (household_id, start_date, seed, name)
        VALUES ($1, $2, $3, $4)
        RETURNING id, household_id, start_date, seed, name, created_at, updated_at
        "#,
        new_plan.household_id,
        new_plan.start_date,
        new_plan.seed,
        new_plan.name,
    )
    .fetch_one(pool)
    .await?;
    Ok(plan)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    plan: UpdateMealPlan,
) -> Result<MealPlan, sqlx::Error> {
    let plan = sqlx::query_as!(
        MealPlan,
        r#"
        UPDATE meal_plan
        SET start_date = COALESCE($1, start_date),
            seed = COALESCE($2, seed),
            name = COALESCE($3, name)
        WHERE id = $4
        RETURNING id, household_id, start_date, seed, name, created_at, updated_at
        "#,
        plan.start_date,
        plan.seed,
        plan.name,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(plan)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<MealPlan, sqlx::Error> {
    let plan = sqlx::query_as!(
        MealPlan,
        r#"
        DELETE FROM meal_plan
        WHERE id = $1
        RETURNING id, household_id, start_date, seed, name, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(plan)
}
