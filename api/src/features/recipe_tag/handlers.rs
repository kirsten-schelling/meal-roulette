use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list};
use crate::error::AppError;
use crate::state::AppState;
use domain::recipe_tag::{NewRecipeTag, RecipeTag};

pub async fn get_recipe_tags(
    State(state): State<AppState>,
) -> Result<Json<Vec<RecipeTag>>, AppError> {
    let links = list(&state.db).await?;
    Ok(Json(links))
}

pub async fn get_recipe_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RecipeTag>, AppError> {
    let link = get(&state.db, id).await?;
    Ok(Json(link))
}

pub async fn create_recipe_tag(
    State(state): State<AppState>,
    Json(new_link): Json<NewRecipeTag>,
) -> Result<Json<RecipeTag>, AppError> {
    let link = create(&state.db, new_link).await?;
    Ok(Json(link))
}

pub async fn delete_recipe_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RecipeTag>, AppError> {
    let link = delete(&state.db, id).await?;
    Ok(Json(link))
}
