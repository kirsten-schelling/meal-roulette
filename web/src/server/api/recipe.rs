use dioxus::prelude::*;
use domain::recipe::{NewRecipe, Recipe};

#[server]
pub async fn get_recipes() -> Result<Vec<Recipe>, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let recipes: Vec<Recipe> = send_request(|c| c.get(get_url("recipes")))
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;
    Ok(recipes)
}

#[server]
pub async fn add_recipe(recipe: NewRecipe) -> Result<Recipe, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let recipe: Recipe = send_request(|c| {
        c.post(get_url("recipes"))
            .body(serde_json::to_string(&recipe).unwrap())
            .header("Content-Type", "application/json")
    })
    .await
    .map_err(|e| ServerFnError::ServerError {
        message: e.to_string(),
        code: 500,
        details: None,
    })?;
    Ok(recipe)
}
