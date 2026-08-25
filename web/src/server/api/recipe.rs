use dioxus::prelude::*;
use domain::recipe::Recipe;

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
