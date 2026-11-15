use dioxus::prelude::*;
use domain::pantry::PantryLine;
use rust_decimal::Decimal;
use uuid::Uuid;

#[server]
pub async fn get_pantry() -> Result<Vec<PantryLine>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::pantry::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

/// Overwrite an ingredient's pantry count.
#[server]
pub async fn set_pantry_qty(
    ingredient_id: Uuid,
    qty: Decimal,
) -> Result<PantryLine, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::pantry::repository;

    let state = app_state()?;
    repository::set_qty(&state.db, ingredient_id, qty)
        .await
        .map_err(server_fn_error)
}
