use dioxus::prelude::*;
use domain::housekeeping::Finding;

#[server]
pub async fn get_housekeeping() -> Result<Vec<Finding>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::housekeeping::service;

    let state = app_state()?;
    service::collect(&state.db).await.map_err(server_fn_error)
}
