use dioxus::prelude::*;
use domain::unit::{NewUnit, Unit, UnitWithIngredientList, UpdateUnit};
use uuid::Uuid;

#[server]
pub async fn get_units() -> Result<Vec<UnitWithIngredientList>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::unit::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_unit(unit: NewUnit) -> Result<Unit, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::unit::repository;

    let state = app_state()?;
    repository::create(&state.db, unit)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn update_unit(id: Uuid, unit: UpdateUnit) -> Result<Unit, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::unit::repository;

    let state = app_state()?;
    repository::update(&state.db, id, unit)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_unit(id: Uuid) -> Result<Unit, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::unit::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}
