use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::shopping_list_item::{NewShoppingListItem, ShoppingListItem, UpdateShoppingListItem};

pub async fn get_shopping_list_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<ShoppingListItem>>, AppError> {
    let items = list(&state.db).await?;
    Ok(Json(items))
}

pub async fn get_shopping_list_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ShoppingListItem>, AppError> {
    let item = get(&state.db, id).await?;
    Ok(Json(item))
}

pub async fn create_shopping_list_item(
    State(state): State<AppState>,
    Json(new_item): Json<NewShoppingListItem>,
) -> Result<Json<ShoppingListItem>, AppError> {
    let item = create(&state.db, new_item).await?;
    Ok(Json(item))
}

pub async fn update_shopping_list_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_item): Json<UpdateShoppingListItem>,
) -> Result<Json<ShoppingListItem>, AppError> {
    let item = update(&state.db, id, update_item).await?;
    Ok(Json(item))
}

pub async fn delete_shopping_list_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ShoppingListItem>, AppError> {
    let item = delete(&state.db, id).await?;
    Ok(Json(item))
}
