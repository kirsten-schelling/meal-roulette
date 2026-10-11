use dioxus::prelude::*;
use domain::meal_plan::MealPlanSummary;

#[server]
pub async fn get_plan_history() -> Result<Vec<MealPlanSummary>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::meal_plan::repository;

    let state = app_state()?;
    repository::list_summaries(&state.db)
        .await
        .map_err(server_fn_error)
}
