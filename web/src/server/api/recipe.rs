use dioxus::prelude::*;
use domain::recipe::{NewRecipe, Recipe, RecipeDetail, RecipeOverview, UpdateRecipe};
use domain::recipe_ingredient::{RecipeIngredient, RecipeIngredientEntry, UpdateRecipeIngredient};
use uuid::Uuid;

#[server]
pub async fn get_recipes() -> Result<Vec<Recipe>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn get_recipe_overviews() -> Result<Vec<RecipeOverview>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    repository::list_overview(&state.db)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn get_recipe_detail(id: Uuid) -> Result<RecipeDetail, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    repository::get_detail(&state.db, id)
        .await
        .map_err(server_fn_error)
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

#[server]
pub async fn update_recipe(id: Uuid, recipe: UpdateRecipe) -> Result<Recipe, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    repository::update(&state.db, id, recipe)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn copy_recipe(id: Uuid, title: String) -> Result<Recipe, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    repository::copy(&state.db, id, &title)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_recipe(id: Uuid) -> Result<Recipe, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn add_recipe_ingredient(
    recipe_id: Uuid,
    line: RecipeIngredientEntry,
) -> Result<RecipeIngredient, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe_ingredient::repository;

    let state = app_state()?;
    repository::add(&state.db, recipe_id, line)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn update_recipe_ingredient(
    recipe_id: Uuid,
    ingredient_id: Uuid,
    line: UpdateRecipeIngredient,
) -> Result<RecipeIngredient, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe_ingredient::repository;

    let state = app_state()?;
    repository::update(&state.db, recipe_id, ingredient_id, line)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn remove_recipe_ingredient(
    recipe_id: Uuid,
    ingredient_id: Uuid,
) -> Result<RecipeIngredient, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::recipe_ingredient::repository;

    let state = app_state()?;
    repository::delete(&state.db, recipe_id, ingredient_id)
        .await
        .map_err(server_fn_error)
}
