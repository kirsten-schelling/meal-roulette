use dioxus::prelude::*;
use domain::cook_log::{CookLog, CookLogEntry};
use uuid::Uuid;

#[server]
pub async fn get_cook_log() -> Result<Vec<CookLogEntry>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cook_log::repository;

    let state = app_state()?;
    repository::list_entries(&state.db)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn get_planned_meal_cook(
    planned_meal_id: Uuid,
) -> Result<Option<CookLog>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cook_log::repository;

    let state = app_state()?;
    repository::find_for_planned_meal(&state.db, planned_meal_id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn log_cook(
    recipe_id: Uuid,
    planned_meal_id: Option<Uuid>,
    cooked_on: time::Date,
) -> Result<CookLog, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cook_log::repository;

    let state = app_state()?;
    repository::log(&state.db, recipe_id, planned_meal_id, cooked_on)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_cook(id: Uuid) -> Result<CookLog, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cook_log::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}
