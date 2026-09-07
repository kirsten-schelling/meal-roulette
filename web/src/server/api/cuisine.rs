use dioxus::prelude::*;
use domain::cuisine::{Cuisine, CuisineWithRecipeList, NewCuisine};

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
