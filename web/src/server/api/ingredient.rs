use dioxus::prelude::*;
use domain::ingredient::{Ingredient, NewIngredient};

#[server]
pub async fn get_ingredients() -> Result<Vec<Ingredient>, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let ingredients: Vec<Ingredient> = send_request(|c| c.get(get_url("ingredients")))
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;
    Ok(ingredients)
}

#[server]
pub async fn add_ingredient(ingredient: NewIngredient) -> Result<Ingredient, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let ingredient: Ingredient = send_request(|c| {
        c.post(get_url("ingredients"))
            .body(serde_json::to_string(&ingredient).unwrap())
    })
    .await
    .map_err(|e| ServerFnError::ServerError {
        message: e.to_string(),
        code: 500,
        details: None,
    })?;
    Ok(ingredient)
}
