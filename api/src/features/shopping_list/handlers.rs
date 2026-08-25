use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::shopping_list::{NewShoppingList, ShoppingList, UpdateShoppingList};

pub async fn get_shopping_lists(
    State(state): State<AppState>,
) -> Result<Json<Vec<ShoppingList>>, AppError> {
    let lists = list(&state.db).await?;
    Ok(Json(lists))
}

pub async fn get_shopping_list(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ShoppingList>, AppError> {
    let list = get(&state.db, id).await?;
    Ok(Json(list))
}

pub async fn create_shopping_list(
    State(state): State<AppState>,
    Json(new_list): Json<NewShoppingList>,
) -> Result<Json<ShoppingList>, AppError> {
    let list = create(&state.db, new_list).await?;
    Ok(Json(list))
}

pub async fn update_shopping_list(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_list): Json<UpdateShoppingList>,
) -> Result<Json<ShoppingList>, AppError> {
    let list = update(&state.db, id, update_list).await?;
    Ok(Json(list))
}

pub async fn delete_shopping_list(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ShoppingList>, AppError> {
    let list = delete(&state.db, id).await?;
    Ok(Json(list))
}
