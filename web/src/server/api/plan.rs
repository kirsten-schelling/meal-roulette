use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;
use domain::planned_meal::PlannedMealDetail;
use uuid::Uuid;

#[server]
pub async fn get_latest_plan() -> Result<Option<MealPlanDetail>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::meal_plan::repository;

    let state = app_state()?;
    repository::latest_detail(&state.db)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn reroll_planned_meal(id: Uuid) -> Result<PlannedMealDetail, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll::service;

    let state = app_state()?;
    service::reroll(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn swap_planned_meal(
    id: Uuid,
    recipe_id: Uuid,
) -> Result<PlannedMealDetail, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::planned_meal::repository;

    let state = app_state()?;
    repository::set_recipe(&state.db, id, recipe_id)
        .await
        .map_err(server_fn_error)
}
