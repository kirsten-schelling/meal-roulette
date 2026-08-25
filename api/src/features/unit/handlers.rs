use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::unit::{NewUnit, Unit, UpdateUnit};

pub async fn get_units(State(state): State<AppState>) -> Result<Json<Vec<Unit>>, AppError> {
    let units = list(&state.db).await?;
    Ok(Json(units))
}

pub async fn get_unit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Unit>, AppError> {
    let unit = get(&state.db, id).await?;
    Ok(Json(unit))
}

pub async fn create_unit(
    State(state): State<AppState>,
    Json(new_unit): Json<NewUnit>,
) -> Result<Json<Unit>, AppError> {
    let unit = create(&state.db, new_unit).await?;
    Ok(Json(unit))
}

pub async fn update_unit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_unit): Json<UpdateUnit>,
) -> Result<Json<Unit>, AppError> {
    let unit = update(&state.db, id, update_unit).await?;
    Ok(Json(unit))
}

pub async fn delete_unit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Unit>, AppError> {
    let unit = delete(&state.db, id).await?;
    Ok(Json(unit))
}
