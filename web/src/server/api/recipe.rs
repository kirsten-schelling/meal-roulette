use dioxus::prelude::*;
use domain::recipe::{NewRecipe, Recipe};

#[server]
pub async fn get_recipes() -> Result<Vec<Recipe>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_recipe(recipe: NewRecipe) -> Result<Recipe, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    let created = repository::create_with_ingredients(&state.db, recipe)
        .await
        .map_err(server_fn_error)?;
    Ok(created.recipe)
}
