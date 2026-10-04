use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use super::service;
use crate::error::AppError;
use crate::state::AppState;
use domain::meal_plan::MealPlanDetail;
use domain::planned_meal::PlannedMealDetail;
use domain::roll::RollRequest;

pub async fn roll(
    State(state): State<AppState>,
    Json(request): Json<RollRequest>,
) -> Result<(StatusCode, Json<MealPlanDetail>), AppError> {
    let plan = service::roll_week(&state.db, request).await?;
    Ok((StatusCode::CREATED, Json(plan)))
}

pub async fn reroll(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlannedMealDetail>, AppError> {
    let meal = service::reroll(&state.db, id).await?;
    Ok(Json(meal))
}
