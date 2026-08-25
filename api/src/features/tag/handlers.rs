use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::tag::{NewTag, Tag, UpdateTag};

pub async fn get_tags(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = list(&state.db).await?;
    Ok(Json(tags))
}

pub async fn get_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Tag>, AppError> {
    let tag = get(&state.db, id).await?;
    Ok(Json(tag))
}

pub async fn create_tag(
    State(state): State<AppState>,
    Json(new_tag): Json<NewTag>,
) -> Result<Json<Tag>, AppError> {
    let tag = create(&state.db, new_tag).await?;
    Ok(Json(tag))
}

pub async fn update_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_tag): Json<UpdateTag>,
) -> Result<Json<Tag>, AppError> {
    let tag = update(&state.db, id, update_tag).await?;
    Ok(Json(tag))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Tag>, AppError> {
    let tag = delete(&state.db, id).await?;
    Ok(Json(tag))
}
