use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::planned_meal::{NewPlannedMeal, PlannedMeal, UpdatePlannedMeal};

pub async fn get_planned_meals(
    State(state): State<AppState>,
) -> Result<Json<Vec<PlannedMeal>>, AppError> {
    let meals = list(&state.db).await?;
    Ok(Json(meals))
}

pub async fn get_planned_meal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlannedMeal>, AppError> {
    let meal = get(&state.db, id).await?;
    Ok(Json(meal))
}

pub async fn create_planned_meal(
    State(state): State<AppState>,
    Json(new_meal): Json<NewPlannedMeal>,
) -> Result<Json<PlannedMeal>, AppError> {
    let meal = create(&state.db, new_meal).await?;
    Ok(Json(meal))
}

pub async fn update_planned_meal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_meal): Json<UpdatePlannedMeal>,
) -> Result<Json<PlannedMeal>, AppError> {
    let meal = update(&state.db, id, update_meal).await?;
    Ok(Json(meal))
}

pub async fn delete_planned_meal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlannedMeal>, AppError> {
    let meal = delete(&state.db, id).await?;
    Ok(Json(meal))
}
