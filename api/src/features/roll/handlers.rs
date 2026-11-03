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
use domain::roll::{
    FitCount, FitRequest, KeepRollRequest, RerollPreview, RerollPreviewRequest, RollOutcome,
    RollPreview, RollRequest,
};

pub async fn roll(
    State(state): State<AppState>,
    Json(request): Json<RollRequest>,
) -> Result<(StatusCode, Json<RollOutcome>), AppError> {
    let outcome = service::roll_week(&state.db, request).await?;
    Ok((StatusCode::CREATED, Json(outcome)))
}

pub async fn reroll(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlannedMealDetail>, AppError> {
    let meal = service::reroll(&state.db, id).await?;
    Ok(Json(meal))
}

pub async fn preview(
    State(state): State<AppState>,
    Json(request): Json<RollRequest>,
) -> Result<Json<RollPreview>, AppError> {
    Ok(Json(service::preview_roll(&state.db, request).await?))
}

pub async fn reroll_preview(
    State(state): State<AppState>,
    Json(request): Json<RerollPreviewRequest>,
) -> Result<Json<RerollPreview>, AppError> {
    Ok(Json(service::reroll_preview(&state.db, request).await?))
}

pub async fn keep(
    State(state): State<AppState>,
    Json(request): Json<KeepRollRequest>,
) -> Result<(StatusCode, Json<MealPlanDetail>), AppError> {
    let plan = service::keep_roll(&state.db, request).await?;
    Ok((StatusCode::CREATED, Json(plan)))
}

pub async fn fit(
    State(state): State<AppState>,
    Json(request): Json<FitRequest>,
) -> Result<Json<FitCount>, AppError> {
    Ok(Json(service::fit_count(&state.db, request).await?))
}
