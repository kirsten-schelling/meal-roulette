use dioxus::prelude::*;
use domain::cuisine::{Cuisine, NewCuisine};

#[server]
pub async fn get_cuisines() -> Result<Vec<Cuisine>, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let cuisines: Vec<Cuisine> =
        send_request(|c| c.get(get_url("cuisines")))
            .await
            .map_err(|e| ServerFnError::ServerError {
                message: e.to_string(),
                code: 500,
                details: None,
            })?;
    Ok(cuisines)
}

#[server]
pub async fn add_cuisine(cuisine: NewCuisine) -> Result<Cuisine, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let cuisine: Cuisine = send_request(|c| {
        c.post(get_url("cuisines"))
            .body(serde_json::to_string(&cuisine).unwrap())
    })
    .await
    .map_err(|e| ServerFnError::ServerError {
        message: e.to_string(),
        code: 500,
        details: None,
    })?;
    Ok(cuisine)
}
