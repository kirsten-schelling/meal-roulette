use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::error::AppError;
use crate::state::AppState;
use domain::protein::{NewProtein, Protein, ProteinWithRecipeList, UpdateProtein};

pub async fn get_proteins(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProteinWithRecipeList>>, AppError> {
    let proteins = list(&state.db).await?;
    Ok(Json(proteins))
}

pub async fn get_protein(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProteinWithRecipeList>, AppError> {
    let protein = get(&state.db, id).await?;
    Ok(Json(protein))
}

pub async fn create_protein(
    State(state): State<AppState>,
    Json(new_protein): Json<NewProtein>,
) -> Result<Json<Protein>, AppError> {
    let protein = create(&state.db, new_protein).await?;
    Ok(Json(protein))
}

pub async fn update_protein(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_protein): Json<UpdateProtein>,
) -> Result<Json<Protein>, AppError> {
    let protein = update(&state.db, id, update_protein).await?;
    Ok(Json(protein))
}

pub async fn delete_protein(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Protein>, AppError> {
    let protein = delete(&state.db, id).await?;
    Ok(Json(protein))
}
