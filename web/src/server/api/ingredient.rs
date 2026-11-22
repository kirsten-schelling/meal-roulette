use dioxus::prelude::*;
use domain::ingredient::{BulkIngredientUpdate, Ingredient, NewIngredient, UpdateIngredient};
use domain::usage::Reference;
use uuid::Uuid;

#[server]
pub async fn get_ingredients() -> Result<Vec<Ingredient>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::ingredient::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_ingredient(ingredient: NewIngredient) -> Result<Ingredient, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::ingredient::repository;

    let state = app_state()?;
    repository::create_with_unit(&state.db, ingredient)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn update_ingredient(
    id: Uuid,
    ingredient: UpdateIngredient,
) -> Result<Ingredient, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::ingredient::repository;

    let state = app_state()?;
    repository::update(&state.db, id, ingredient)
        .await
        .map_err(server_fn_error)
}

/// Apply the same changes to every ingredient in `ids`; returns how many were changed.
#[server]
pub async fn update_ingredients(
    ids: Vec<Uuid>,
    changes: BulkIngredientUpdate,
) -> Result<u64, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::ingredient::repository;

    let state = app_state()?;
    repository::update_many(&state.db, &ids, changes)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_ingredient(id: Uuid) -> Result<Ingredient, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::ingredient::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn get_ingredient_references(id: Uuid) -> Result<Vec<Reference>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::ingredient::repository;

    let state = app_state()?;
    repository::references(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_ingredient_cascade(id: Uuid) -> Result<Ingredient, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::ingredient::repository;

    let state = app_state()?;
    repository::delete_with_references(&state.db, id)
        .await
        .map_err(server_fn_error)
}
