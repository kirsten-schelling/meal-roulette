use dioxus::prelude::*;
use domain::ingredient::{Ingredient, NewIngredient};

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
