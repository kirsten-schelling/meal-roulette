use dioxus::prelude::*;
use domain::shopping_item::ShoppingItem;
use uuid::Uuid;

#[server]
pub async fn get_shopping_items() -> Result<Vec<ShoppingItem>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_item::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_shopping_item(
    name: String,
    shopping_category_id: Option<Uuid>,
) -> Result<ShoppingItem, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_item::repository;

    let state = app_state()?;
    repository::create(&state.db, &name, shopping_category_id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn update_shopping_item(
    id: Uuid,
    name: String,
    shopping_category_id: Option<Uuid>,
) -> Result<ShoppingItem, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_item::repository;

    let state = app_state()?;
    repository::update(&state.db, id, &name, shopping_category_id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_shopping_item(id: Uuid) -> Result<ShoppingItem, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::shopping_item::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}
