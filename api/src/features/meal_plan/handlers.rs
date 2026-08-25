use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::meal_plan::{MealPlan, NewMealPlan, UpdateMealPlan};

pub async fn get_meal_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<MealPlan>>, AppError> {
    let plans = list(&state.db).await?;
    Ok(Json(plans))
}

pub async fn get_meal_plan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MealPlan>, AppError> {
    let plan = get(&state.db, id).await?;
    Ok(Json(plan))
}

pub async fn create_meal_plan(
    State(state): State<AppState>,
    Json(new_plan): Json<NewMealPlan>,
) -> Result<Json<MealPlan>, AppError> {
    let plan = create(&state.db, new_plan).await?;
    Ok(Json(plan))
}

pub async fn update_meal_plan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_plan): Json<UpdateMealPlan>,
) -> Result<Json<MealPlan>, AppError> {
    let plan = update(&state.db, id, update_plan).await?;
    Ok(Json(plan))
}

pub async fn delete_meal_plan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MealPlan>, AppError> {
    let plan = delete(&state.db, id).await?;
    Ok(Json(plan))
}
