use dioxus::prelude::*;
use domain::roll_preset::{NewRollPreset, RollPreset, UpdateRollPreset};
use uuid::Uuid;

#[server]
pub async fn get_roll_presets() -> Result<Vec<RollPreset>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll_preset::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_roll_preset(preset: NewRollPreset) -> Result<RollPreset, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll_preset::repository;

    let state = app_state()?;
    repository::create(&state.db, preset)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn update_roll_preset(
    id: Uuid,
    preset: UpdateRollPreset,
) -> Result<RollPreset, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll_preset::repository;

    let state = app_state()?;
    repository::update(&state.db, id, preset)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn set_default_roll_preset(id: Uuid) -> Result<RollPreset, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll_preset::repository;

    let state = app_state()?;
    repository::set_default(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_roll_preset(id: Uuid) -> Result<RollPreset, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll_preset::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}
