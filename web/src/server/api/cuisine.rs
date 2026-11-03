use dioxus::prelude::*;
use domain::cuisine::{Cuisine, CuisineWithRecipeList, NewCuisine, UpdateCuisine};
use domain::usage::Reference;
use uuid::Uuid;

#[server]
pub async fn get_cuisines() -> Result<Vec<CuisineWithRecipeList>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cuisine::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_cuisine(cuisine: NewCuisine) -> Result<Cuisine, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cuisine::repository;

    let state = app_state()?;
    repository::create(&state.db, cuisine)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn update_cuisine(id: Uuid, cuisine: UpdateCuisine) -> Result<Cuisine, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cuisine::repository;

    let state = app_state()?;
    repository::update(&state.db, id, cuisine)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_cuisine(id: Uuid) -> Result<Cuisine, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cuisine::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn get_cuisine_references(id: Uuid) -> Result<Vec<Reference>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cuisine::repository;

    let state = app_state()?;
    repository::references(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_cuisine_cascade(id: Uuid) -> Result<Cuisine, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::cuisine::repository;

    let state = app_state()?;
    repository::delete_detaching(&state.db, id)
        .await
        .map_err(server_fn_error)
}
