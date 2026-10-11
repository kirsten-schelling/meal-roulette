use dioxus::prelude::*;
use domain::meal_plan::{MealPlan, MealPlanDetail};
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
pub async fn get_plan(id: Uuid) -> Result<MealPlanDetail, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::meal_plan::repository;

    let state = app_state()?;
    repository::get_detail(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn set_plan_favourite(id: Uuid, is_favourite: bool) -> Result<MealPlan, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::meal_plan::repository;

    let state = app_state()?;
    repository::set_favourite(&state.db, id, is_favourite)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn set_plan_archived(id: Uuid, archived: bool) -> Result<MealPlan, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::meal_plan::repository;

    let state = app_state()?;
    repository::set_archived(&state.db, id, archived)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_plan(id: Uuid) -> Result<MealPlan, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::meal_plan::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
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
