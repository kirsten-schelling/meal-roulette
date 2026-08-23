use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::model::{NewRecipe, Recipe, UpdateRecipe};
use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;

pub async fn get_recipes(State(state): State<AppState>) -> Result<Json<Vec<Recipe>>, AppError> {
    let recipes = list(&state.db).await?;
    Ok(Json(recipes))
}

pub async fn get_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = get(&state.db, id).await?;
    Ok(Json(recipe))
}
pub async fn create_recipe(
    State(state): State<AppState>,
    Json(new_recipe): Json<NewRecipe>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = create(&state.db, new_recipe).await?;
    Ok(Json(recipe))
}

pub async fn update_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_recipe): Json<UpdateRecipe>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = update(&state.db, id, update_recipe).await?;
    Ok(Json(recipe))
}

pub async fn delete_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = delete(&state.db, id).await?;
    Ok(Json(recipe))
}
