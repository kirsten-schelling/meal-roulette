use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;
use domain::roll::RollRequest;

#[server]
pub async fn roll_week(request: RollRequest) -> Result<MealPlanDetail, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll::service;

    let state = app_state()?;
    service::roll_week(&state.db, request)
        .await
        .map_err(server_fn_error)
}
