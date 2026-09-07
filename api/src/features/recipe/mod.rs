mod handlers;
pub mod repository;

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
            get(handlers::get_recipe)
                .put(handlers::update_recipe)
                .delete(handlers::delete_recipe),
        )
        .route(
            "/recipes/cuisine/{cuisine_id}",
            get(handlers::get_recipes_by_cuisine),
        )
}
