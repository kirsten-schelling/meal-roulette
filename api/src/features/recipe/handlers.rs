use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create_with_ingredients, delete, get, list, list_by_cuisine, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::recipe::{NewRecipe, Recipe, RecipeWithIngredients, UpdateRecipe};

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
) -> Result<Json<RecipeWithIngredients>, AppError> {
    let recipe = create_with_ingredients(&state.db, new_recipe).await?;
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

pub async fn get_recipes_by_cuisine(
    State(state): State<AppState>,
    Path(cuisine_id): Path<Uuid>,
) -> Result<Json<Vec<Recipe>>, AppError> {
    let recipes = list_by_cuisine(&state.db, cuisine_id).await?;
    Ok(Json(recipes))
}
