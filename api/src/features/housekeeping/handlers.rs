use axum::{extract::State, Json};

use super::service::collect;
use crate::error::AppError;
use crate::state::AppState;
use domain::housekeeping::Finding;

pub async fn get_housekeeping(
    State(state): State<AppState>,
) -> Result<Json<Vec<Finding>>, AppError> {
    Ok(Json(collect(&state.db).await?))
}
