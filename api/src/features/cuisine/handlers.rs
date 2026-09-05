use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::{is_foreign_key_violation, AppError};
use crate::state::AppState;
use domain::cuisine::{Cuisine, NewCuisine, UpdateCuisine};

pub async fn get_cuisines(State(state): State<AppState>) -> Result<Json<Vec<Cuisine>>, AppError> {
    let cuisines = list(&state.db).await?;
    Ok(Json(cuisines))
}

pub async fn get_cuisine(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Cuisine>, AppError> {
    let cuisine = get(&state.db, id).await?;
    Ok(Json(cuisine))
}

pub async fn create_cuisine(
    State(state): State<AppState>,
    Json(new_cuisine): Json<NewCuisine>,
) -> Result<Json<Cuisine>, AppError> {
    let cuisine = create(&state.db, new_cuisine).await?;
    Ok(Json(cuisine))
}

pub async fn update_cuisine(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_cuisine): Json<UpdateCuisine>,
) -> Result<Json<Cuisine>, AppError> {
    let cuisine = update(&state.db, id, update_cuisine).await?;
    Ok(Json(cuisine))
}

pub async fn delete_cuisine(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Cuisine>, AppError> {
    let cuisine = delete(&state.db, id).await.map_err(|e| {
        if let Some(_db) = is_foreign_key_violation(&e) {
            return AppError::Conflict("Still in use by a recipe".to_string());
        }
        e.into()
    })?;
    Ok(Json(cuisine))
}
