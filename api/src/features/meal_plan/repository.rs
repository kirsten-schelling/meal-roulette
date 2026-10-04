use crate::features::planned_meal;
use domain::meal_plan::{MealPlan, MealPlanDetail, NewMealPlan, UpdateMealPlan};
use domain::roll::RollConstraints;
use sqlx::types::Json;
use sqlx::PgPool;
use uuid::Uuid;

struct MealPlanRow {
    id: Uuid,
    household_id: Uuid,
    start_date: time::Date,
    seed: Option<i64>,
    name: Option<String>,
    roll_constraints: Json<RollConstraints>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

impl From<MealPlanRow> for MealPlan {
    fn from(row: MealPlanRow) -> Self {
        MealPlan {
            id: row.id,
            household_id: row.household_id,
            start_date: row.start_date,
            seed: row.seed,
            name: row.name,
            constraints: row.roll_constraints.0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub async fn list(pool: &PgPool) -> Result<Vec<MealPlan>, sqlx::Error> {
    let rows = sqlx::query_as!(
        MealPlanRow,
        r#"
        SELECT id, household_id, start_date, seed, name,
               roll_constraints AS "roll_constraints: Json<RollConstraints>",
               created_at, updated_at
        FROM meal_plan
        ORDER BY start_date DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<MealPlan, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        SELECT id, household_id, start_date, seed, name,
               roll_constraints AS "roll_constraints: Json<RollConstraints>",
               created_at, updated_at
        FROM meal_plan
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

/// The most recently rolled plan, if any plan exists at all.
pub async fn latest(pool: &PgPool) -> Result<Option<MealPlan>, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        SELECT id, household_id, start_date, seed, name,
               roll_constraints AS "roll_constraints: Json<RollConstraints>",
               created_at, updated_at
        FROM meal_plan
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn get_detail(pool: &PgPool, id: Uuid) -> Result<MealPlanDetail, sqlx::Error> {
    let plan = get(pool, id).await?;
    let meals = planned_meal::repository::list_detail(pool, plan.id).await?;
    Ok(MealPlanDetail { plan, meals })
}

pub async fn latest_detail(pool: &PgPool) -> Result<Option<MealPlanDetail>, sqlx::Error> {
    let Some(plan) = latest(pool).await? else {
        return Ok(None);
    };
    let meals = planned_meal::repository::list_detail(pool, plan.id).await?;
    Ok(Some(MealPlanDetail { plan, meals }))
}

pub async fn create<'e, E>(executor: E, new_plan: NewMealPlan) -> Result<MealPlan, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        INSERT INTO meal_plan (household_id, start_date, seed, name, roll_constraints)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, household_id, start_date, seed, name,
                  roll_constraints AS "roll_constraints: Json<RollConstraints>",
                  created_at, updated_at
        "#,
        new_plan.household_id,
        new_plan.start_date,
        new_plan.seed,
        new_plan.name,
        Json(new_plan.constraints) as Json<RollConstraints>,
    )
    .fetch_one(executor)
    .await?;
    Ok(row.into())
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    plan: UpdateMealPlan,
) -> Result<MealPlan, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        UPDATE meal_plan
        SET start_date = COALESCE($1, start_date),
            seed = COALESCE($2, seed),
            name = COALESCE($3, name)
        WHERE id = $4
        RETURNING id, household_id, start_date, seed, name,
                  roll_constraints AS "roll_constraints: Json<RollConstraints>",
                  created_at, updated_at
        "#,
        plan.start_date,
        plan.seed,
        plan.name,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<MealPlan, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        DELETE FROM meal_plan
        WHERE id = $1
        RETURNING id, household_id, start_date, seed, name,
                  roll_constraints AS "roll_constraints: Json<RollConstraints>",
                  created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}
