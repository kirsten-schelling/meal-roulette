use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::model::{NewRecipeIngredient, RecipeIngredient, UpdateRecipeIngredient};
use super::repository::{create, delete, get, list_for_recipe, update};
use crate::error::AppError;
use crate::state::AppState;

pub async fn list_recipe_ingredients(
    State(state): State<AppState>,
    Path(recipe_id): Path<Uuid>,
) -> Result<Json<Vec<RecipeIngredient>>, AppError> {
    let items = list_for_recipe(&state.db, recipe_id).await?;
    Ok(Json(items))
}

pub async fn get_recipe_ingredient(
    State(state): State<AppState>,
    Path((recipe_id, ingredient_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RecipeIngredient>, AppError> {
    let item = get(&state.db, recipe_id, ingredient_id).await?;
    Ok(Json(item))
}

pub async fn add_recipe_ingredient(
    State(state): State<AppState>,
    Path(recipe_id): Path<Uuid>,
    Json(new): Json<NewRecipeIngredient>,
) -> Result<Json<RecipeIngredient>, AppError> {
    let item = create(&state.db, recipe_id, new).await?;
    Ok(Json(item))
}

pub async fn update_recipe_ingredient(
    State(state): State<AppState>,
    Path((recipe_id, ingredient_id)): Path<(Uuid, Uuid)>,
    Json(update_body): Json<UpdateRecipeIngredient>,
) -> Result<Json<RecipeIngredient>, AppError> {
    let item = update(&state.db, recipe_id, ingredient_id, update_body).await?;
    Ok(Json(item))
}

pub async fn delete_recipe_ingredient(
    State(state): State<AppState>,
    Path((recipe_id, ingredient_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RecipeIngredient>, AppError> {
    let item = delete(&state.db, recipe_id, ingredient_id).await?;
    Ok(Json(item))
}
