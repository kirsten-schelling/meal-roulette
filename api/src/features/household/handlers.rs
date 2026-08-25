use super::repository::{create, delete, get, list, update};
use crate::{error::AppError, state::AppState};
use axum::extract::Path;
use axum::{extract::State, Json};
use domain::household::{Household, NewHousehold, UpdateHousehold};
use uuid::Uuid;

pub async fn get_households(
    State(state): State<AppState>,
) -> Result<Json<Vec<Household>>, AppError> {
    let households = list(&state.db).await?;
    Ok(Json(households))
}

pub async fn get_household(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Household>, AppError> {
    let household = get(&state.db, id).await?;
    Ok(Json(household))
}

pub async fn create_household(
    State(state): State<AppState>,
    Json(new_household): Json<NewHousehold>,
) -> Result<Json<Household>, AppError> {
    let household = create(&state.db, new_household).await?;
    Ok(Json(household))
}

pub async fn update_household(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_household): Json<UpdateHousehold>,
) -> Result<Json<Household>, AppError> {
    let household = update(&state.db, id, update_household).await?;
    Ok(Json(household))
}

pub async fn delete_household(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Household>, AppError> {
    let household = delete(&state.db, id).await?;
    Ok(Json(household))
}
