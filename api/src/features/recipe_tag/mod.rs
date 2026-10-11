mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/recipe-tags",
            get(handlers::get_recipe_tags).post(handlers::create_recipe_tag),
        )
        .route(
            "/recipe-tags/{id}",
            get(handlers::get_recipe_tag).delete(handlers::delete_recipe_tag),
        )
}
