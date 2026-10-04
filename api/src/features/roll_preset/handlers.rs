use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, set_default, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::roll_preset::{NewRollPreset, RollPreset, UpdateRollPreset};

pub async fn get_roll_presets(
    State(state): State<AppState>,
) -> Result<Json<Vec<RollPreset>>, AppError> {
    let presets = list(&state.db).await?;
    Ok(Json(presets))
}

pub async fn get_roll_preset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RollPreset>, AppError> {
    let preset = get(&state.db, id).await?;
    Ok(Json(preset))
}

pub async fn create_roll_preset(
    State(state): State<AppState>,
    Json(new_preset): Json<NewRollPreset>,
) -> Result<Json<RollPreset>, AppError> {
    let preset = create(&state.db, new_preset).await?;
    Ok(Json(preset))
}

pub async fn update_roll_preset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_preset): Json<UpdateRollPreset>,
) -> Result<Json<RollPreset>, AppError> {
    let preset = update(&state.db, id, update_preset).await?;
    Ok(Json(preset))
}

pub async fn set_default_roll_preset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RollPreset>, AppError> {
    let preset = set_default(&state.db, id).await?;
    Ok(Json(preset))
}

pub async fn delete_roll_preset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RollPreset>, AppError> {
    let preset = delete(&state.db, id).await?;
    Ok(Json(preset))
}
