mod handlers;
mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/recipes/{id}/ingredients",
            get(handlers::list_recipe_ingredients).post(handlers::add_recipe_ingredient),
        )
        .route(
            "/recipes/{id}/ingredients/{ingredient_id}",
            get(handlers::get_recipe_ingredient)
                .put(handlers::update_recipe_ingredient)
                .delete(handlers::delete_recipe_ingredient),
        )
}
