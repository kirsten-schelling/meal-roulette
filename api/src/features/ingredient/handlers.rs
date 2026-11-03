use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{
    create_with_unit, delete, delete_with_references, get, list, references, update,
};
use crate::error::AppError;
use crate::state::AppState;
use domain::ingredient::{Ingredient, NewIngredient, UpdateIngredient};
use domain::usage::Reference;

pub async fn get_ingredients(
    State(state): State<AppState>,
) -> Result<Json<Vec<Ingredient>>, AppError> {
    let ingredients = list(&state.db).await?;
    Ok(Json(ingredients))
}

pub async fn get_ingredient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Ingredient>, AppError> {
    let ingredient = get(&state.db, id).await?;
    Ok(Json(ingredient))
}

pub async fn create_ingredient(
    State(state): State<AppState>,
    Json(new_ingredient): Json<NewIngredient>,
) -> Result<Json<Ingredient>, AppError> {
    let ingredient = create_with_unit(&state.db, new_ingredient).await?;
    Ok(Json(ingredient))
}

pub async fn update_ingredient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_ingredient): Json<UpdateIngredient>,
) -> Result<Json<Ingredient>, AppError> {
    let ingredient = update(&state.db, id, update_ingredient).await?;
    Ok(Json(ingredient))
}

pub async fn delete_ingredient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Ingredient>, AppError> {
    let ingredient = delete(&state.db, id).await?;
    Ok(Json(ingredient))
}

pub async fn get_ingredient_references(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Reference>>, AppError> {
    let found = references(&state.db, id).await?;
    Ok(Json(found))
}

pub async fn delete_ingredient_cascade(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Ingredient>, AppError> {
    let deleted = delete_with_references(&state.db, id).await?;
    Ok(Json(deleted))
}
