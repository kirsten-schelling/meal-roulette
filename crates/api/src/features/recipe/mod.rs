mod handlers;
mod model;
mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/recipes",
            get(handlers::get_recipes).post(handlers::create_recipe),
        )
        .route(
            "/recipes/{id}",
            get(handlers::get_recipes).put(handlers::update_recipe),
        )
}
