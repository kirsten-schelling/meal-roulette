use crate::features::planned_meal;
use domain::meal_plan::{MealPlan, MealPlanDetail, MealPlanSummary, NewMealPlan, UpdateMealPlan};
use domain::roll::RollConstraints;
use sqlx::types::Json;
use sqlx::PgPool;
use uuid::Uuid;

struct MealPlanRow {
    id: Uuid,
    household_id: Uuid,
    start_date: Option<time::Date>,
    seed: Option<i64>,
    name: Option<String>,
    roll_constraints: Json<RollConstraints>,
    is_favourite: bool,
    archived_at: Option<time::OffsetDateTime>,
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
            is_favourite: row.is_favourite,
            archived_at: row.archived_at,
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
               is_favourite, archived_at, created_at, updated_at
        FROM meal_plan
        ORDER BY created_at DESC
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
               is_favourite, archived_at, created_at, updated_at
        FROM meal_plan
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

/// The most recently rolled plan that is not archived, if there is one.
pub async fn latest(pool: &PgPool) -> Result<Option<MealPlan>, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        SELECT id, household_id, start_date, seed, name,
               roll_constraints AS "roll_constraints: Json<RollConstraints>",
               is_favourite, archived_at, created_at, updated_at
        FROM meal_plan
        WHERE archived_at IS NULL
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

/// Every plan, newest roll first, with its dinner titles in roll order.
pub async fn list_summaries(pool: &PgPool) -> Result<Vec<MealPlanSummary>, sqlx::Error> {
    sqlx::query_as!(
        MealPlanSummary,
        r#"
        SELECT meal_plan.id, meal_plan.household_id, meal_plan.created_at, meal_plan.name,
               COUNT(planned_meal.id) AS "num_days!",
               meal_plan.seed, meal_plan.is_favourite, meal_plan.archived_at,
               COALESCE(
                   ARRAY_AGG(recipe.title ORDER BY planned_meal.day_index)
                       FILTER (WHERE planned_meal.id IS NOT NULL),
                   ARRAY[]::text[]
               ) AS "dinners!"
        FROM meal_plan
        LEFT JOIN planned_meal ON planned_meal.meal_plan_id = meal_plan.id
        LEFT JOIN recipe ON recipe.id = planned_meal.recipe_id
        GROUP BY meal_plan.id
        ORDER BY meal_plan.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_summary(pool: &PgPool, id: Uuid) -> Result<MealPlanSummary, sqlx::Error> {
    sqlx::query_as!(
        MealPlanSummary,
        r#"
        SELECT meal_plan.id, meal_plan.household_id, meal_plan.created_at, meal_plan.name,
               COUNT(planned_meal.id) AS "num_days!",
               meal_plan.seed, meal_plan.is_favourite, meal_plan.archived_at,
               COALESCE(
                   ARRAY_AGG(recipe.title ORDER BY planned_meal.day_index)
                       FILTER (WHERE planned_meal.id IS NOT NULL),
                   ARRAY[]::text[]
               ) AS "dinners!"
        FROM meal_plan
        LEFT JOIN planned_meal ON planned_meal.meal_plan_id = meal_plan.id
        LEFT JOIN recipe ON recipe.id = planned_meal.recipe_id
        WHERE meal_plan.id = $1
        GROUP BY meal_plan.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await
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
                  is_favourite, archived_at, created_at, updated_at
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
                  is_favourite, archived_at, created_at, updated_at
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

pub async fn set_favourite(
    pool: &PgPool,
    id: Uuid,
    is_favourite: bool,
) -> Result<MealPlan, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        UPDATE meal_plan
        SET is_favourite = $1
        WHERE id = $2
        RETURNING id, household_id, start_date, seed, name,
                  roll_constraints AS "roll_constraints: Json<RollConstraints>",
                  is_favourite, archived_at, created_at, updated_at
        "#,
        is_favourite,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

/// Archive or restore a plan. Archiving an already archived plan keeps its
/// original archive time.
pub async fn set_archived(
    pool: &PgPool,
    id: Uuid,
    archived: bool,
) -> Result<MealPlan, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        UPDATE meal_plan
        SET archived_at = CASE WHEN $1 THEN COALESCE(archived_at, now()) END
        WHERE id = $2
        RETURNING id, household_id, start_date, seed, name,
                  roll_constraints AS "roll_constraints: Json<RollConstraints>",
                  is_favourite, archived_at, created_at, updated_at
        "#,
        archived,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

/// Delete a plan. Its days go with it; its shopping list and any cooking-log
/// entries stay, detached from the plan by their foreign keys.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<MealPlan, sqlx::Error> {
    let row = sqlx::query_as!(
        MealPlanRow,
        r#"
        DELETE FROM meal_plan
        WHERE id = $1
        RETURNING id, household_id, start_date, seed, name,
                  roll_constraints AS "roll_constraints: Json<RollConstraints>",
                  is_favourite, archived_at, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

/// Put the plan's dinners on consecutive dates from `start_date`, in roll order,
/// replacing any dates they had.
pub async fn schedule(
    pool: &PgPool,
    id: Uuid,
    start_date: time::Date,
) -> Result<MealPlanDetail, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query_scalar!(
        "UPDATE meal_plan SET start_date = $2 WHERE id = $1 RETURNING id",
        id,
        start_date,
    )
    .fetch_one(&mut *tx)
    .await?;
    // A dinner's new date can be another's old one, which the unique constraint
    // would refuse row by row, so clear them all first.
    let _cleared = sqlx::query_scalar!(
        "UPDATE planned_meal SET scheduled_date = NULL WHERE meal_plan_id = $1 RETURNING id",
        id,
    )
    .fetch_all(&mut *tx)
    .await?;
    let _placed = sqlx::query_scalar!(
        "UPDATE planned_meal SET scheduled_date = $2::date + (day_index - 1)::int WHERE meal_plan_id = $1 RETURNING id",
        id,
        start_date,
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    get_detail(pool, id).await
}
