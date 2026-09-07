use dioxus::prelude::*;
use domain::unit::{NewUnit, Unit, UnitWithIngredientList};

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
