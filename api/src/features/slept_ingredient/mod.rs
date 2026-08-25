mod handlers;
mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/slept-ingredients",
            get(handlers::get_slept_ingredients).post(handlers::create_slept_ingredient),
        )
        .route(
            "/slept-ingredients/{id}",
            get(handlers::get_slept_ingredient)
                .put(handlers::update_slept_ingredient)
                .delete(handlers::delete_slept_ingredient),
        )
}
