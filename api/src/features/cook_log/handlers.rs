use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::cook_log::{CookLog, NewCookLog, UpdateCookLog};

pub async fn get_cook_logs(State(state): State<AppState>) -> Result<Json<Vec<CookLog>>, AppError> {
    let logs = list(&state.db).await?;
    Ok(Json(logs))
}

pub async fn get_cook_log(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CookLog>, AppError> {
    let log = get(&state.db, id).await?;
    Ok(Json(log))
}

pub async fn create_cook_log(
    State(state): State<AppState>,
    Json(new_log): Json<NewCookLog>,
) -> Result<Json<CookLog>, AppError> {
    let log = create(&state.db, new_log).await?;
    Ok(Json(log))
}

pub async fn update_cook_log(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_log): Json<UpdateCookLog>,
) -> Result<Json<CookLog>, AppError> {
    let log = update(&state.db, id, update_log).await?;
    Ok(Json(log))
}

pub async fn delete_cook_log(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CookLog>, AppError> {
    let log = delete(&state.db, id).await?;
    Ok(Json(log))
}
