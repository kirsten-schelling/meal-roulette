use domain::planned_meal::{NewPlannedMeal, PlannedMeal, PlannedMealDetail, UpdatePlannedMeal};
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

/// Every slot of one plan with its recipe, in date order.
pub async fn list_detail(
    pool: &PgPool,
    meal_plan_id: Uuid,
) -> Result<Vec<PlannedMealDetail>, sqlx::Error> {
    let meals = sqlx::query_as!(
        PlannedMealDetail,
        r#"
        SELECT planned_meal.id, planned_meal.meal_plan_id, planned_meal.recipe_id,
               planned_meal.scheduled_date, planned_meal.is_pinned, planned_meal.target_difficulty,
               recipe.title, recipe.difficulty, recipe.time_mins, recipe.servings, recipe.healthiness,
               recipe.cuisine_id, cuisine.name AS "cuisine?",
               recipe.main_protein_id, protein.name AS "protein?"
        FROM planned_meal
        JOIN recipe ON recipe.id = planned_meal.recipe_id
        LEFT JOIN cuisine ON cuisine.id = recipe.cuisine_id
        LEFT JOIN protein ON protein.id = recipe.main_protein_id
        WHERE planned_meal.meal_plan_id = $1
        ORDER BY planned_meal.scheduled_date
        "#,
        meal_plan_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(meals)
}

pub async fn get_detail(pool: &PgPool, id: Uuid) -> Result<PlannedMealDetail, sqlx::Error> {
    let meal = sqlx::query_as!(
        PlannedMealDetail,
        r#"
        SELECT planned_meal.id, planned_meal.meal_plan_id, planned_meal.recipe_id,
               planned_meal.scheduled_date, planned_meal.is_pinned, planned_meal.target_difficulty,
               recipe.title, recipe.difficulty, recipe.time_mins, recipe.servings, recipe.healthiness,
               recipe.cuisine_id, cuisine.name AS "cuisine?",
               recipe.main_protein_id, protein.name AS "protein?"
        FROM planned_meal
        JOIN recipe ON recipe.id = planned_meal.recipe_id
        LEFT JOIN cuisine ON cuisine.id = recipe.cuisine_id
        LEFT JOIN protein ON protein.id = recipe.main_protein_id
        WHERE planned_meal.id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(meal)
}

pub async fn create<'e, E>(
    executor: E,
    new_meal: NewPlannedMeal,
) -> Result<PlannedMeal, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
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
    .fetch_one(executor)
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

/// Put a different recipe in a slot and return the slot as the plan page shows it.
pub async fn set_recipe(
    pool: &PgPool,
    id: Uuid,
    recipe_id: Uuid,
) -> Result<PlannedMealDetail, sqlx::Error> {
    update(
        pool,
        id,
        UpdatePlannedMeal {
            recipe_id: Some(recipe_id),
            ..UpdatePlannedMeal::default()
        },
    )
    .await?;
    get_detail(pool, id).await
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
