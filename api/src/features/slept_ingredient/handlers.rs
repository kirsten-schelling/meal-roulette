use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::slept_ingredient::{NewSleptIngredient, SleptIngredient, UpdateSleptIngredient};

pub async fn get_slept_ingredients(
    State(state): State<AppState>,
) -> Result<Json<Vec<SleptIngredient>>, AppError> {
    let slept = list(&state.db).await?;
    Ok(Json(slept))
}

pub async fn get_slept_ingredient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SleptIngredient>, AppError> {
    let slept = get(&state.db, id).await?;
    Ok(Json(slept))
}

pub async fn create_slept_ingredient(
    State(state): State<AppState>,
    Json(new_slept): Json<NewSleptIngredient>,
) -> Result<Json<SleptIngredient>, AppError> {
    let slept = create(&state.db, new_slept).await?;
    Ok(Json(slept))
}

pub async fn update_slept_ingredient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_slept): Json<UpdateSleptIngredient>,
) -> Result<Json<SleptIngredient>, AppError> {
    let slept = update(&state.db, id, update_slept).await?;
    Ok(Json(slept))
}

pub async fn delete_slept_ingredient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SleptIngredient>, AppError> {
    let slept = delete(&state.db, id).await?;
    Ok(Json(slept))
}
