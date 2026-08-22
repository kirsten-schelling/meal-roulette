use crate::models::Recipe;
use dioxus::prelude::*;

// The bodies of these #[server] functions run ONLY on the server (the BFF).
// The client gets a typed stub that calls them over HTTP. This is where an
// OIDC access token would be attached before calling the Rust API.

#[server]
pub async fn list_recipes() -> Result<Vec<Recipe>, ServerFnError> {
    let recipes = async {
        let resp = reqwest::get("http://localhost:3000/recipes").await.ok()?;
        resp.json::<Vec<Recipe>>().await.ok()
    }
    .await
    .unwrap_or_default();
    Ok(recipes)
}

#[server]
pub async fn add_recipe(
    title: String,
    difficulty: i32,
    time_mins: i32,
) -> Result<(), ServerFnError> {
    let body = serde_json::json!({
        "title": title,
        "difficulty": difficulty,
        "time_mins": time_mins,
    });
    let _ = reqwest::Client::new()
        .post("http://localhost:3000/recipes")
        .json(&body)
        .send()
        .await;
    Ok(())
}
