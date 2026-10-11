use dioxus::prelude::*;
use domain::shopping_list::{ShoppingList, ShoppingListForPlan};
use domain::shopping_list_item::ShoppingListItem;
use domain::slept_ingredient::SleptIngredient;
use uuid::Uuid;

#[server]
pub async fn get_latest_shopping_list() -> Result<Option<ShoppingListForPlan>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_list::repository;

    let state = app_state()?;
    repository::for_latest_plan(&state.db)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn get_shopping_list(plan_id: Uuid) -> Result<ShoppingListForPlan, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_list::repository;

    let state = app_state()?;
    repository::for_plan(&state.db, plan_id)
        .await
        .map_err(server_fn_error)
}

/// Build the plan's list, or rebuild the one it has.
#[server]
pub async fn generate_shopping_list(plan_id: Uuid) -> Result<ShoppingList, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_list::repository;

    let state = app_state()?;
    repository::generate_for_plan(&state.db, plan_id)
        .await
        .map_err(server_fn_error)
}

/// The plan's list, built only if the plan has none yet.
#[server]
pub async fn ensure_shopping_list(plan_id: Uuid) -> Result<ShoppingList, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_list::repository;

    let state = app_state()?;
    repository::ensure_for_plan(&state.db, plan_id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn set_item_purchased(
    id: Uuid,
    is_purchased: bool,
) -> Result<ShoppingListItem, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_list_item::repository;

    let state = app_state()?;
    repository::set_purchased(&state.db, id, is_purchased)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn sleep_ingredients(
    household_id: Uuid,
    ingredient_ids: Vec<Uuid>,
    weeks: i32,
) -> Result<Vec<SleptIngredient>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::slept_ingredient::repository;

    let state = app_state()?;
    repository::sleep_for_weeks(&state.db, household_id, &ingredient_ids, weeks)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn wake_ingredient(
    household_id: Uuid,
    ingredient_id: Uuid,
) -> Result<Option<SleptIngredient>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::slept_ingredient::repository;

    let state = app_state()?;
    repository::wake(&state.db, household_id, ingredient_id)
        .await
        .map_err(server_fn_error)
}
