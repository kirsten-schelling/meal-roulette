use dioxus::prelude::*;
use domain::shopping_category::ShoppingCategory;
use uuid::Uuid;

#[server]
pub async fn get_shopping_categories() -> Result<Vec<ShoppingCategory>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_category::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_shopping_category(name: String) -> Result<ShoppingCategory, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_category::repository;

    let state = app_state()?;
    repository::create(&state.db, &name)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn rename_shopping_category(
    id: Uuid,
    name: String,
) -> Result<ShoppingCategory, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_category::repository;

    let state = app_state()?;
    repository::rename(&state.db, id, &name)
        .await
        .map_err(server_fn_error)
}

/// Put the categories in the order `ids` gives them.
#[server]
pub async fn reorder_shopping_categories(
    ids: Vec<Uuid>,
) -> Result<Vec<ShoppingCategory>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_category::repository;

    let state = app_state()?;
    repository::reorder(&state.db, &ids)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_shopping_category(id: Uuid) -> Result<ShoppingCategory, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_category::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}
