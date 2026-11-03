mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{
    routing::{delete, get},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/ingredients",
            get(handlers::get_ingredients).post(handlers::create_ingredient),
        )
        .route(
            "/ingredients/{id}",
            get(handlers::get_ingredient)
                .put(handlers::update_ingredient)
                .delete(handlers::delete_ingredient),
        )
        .route(
            "/ingredients/{id}/references",
            get(handlers::get_ingredient_references),
        )
        .route(
            "/ingredients/{id}/cascade",
            delete(handlers::delete_ingredient_cascade),
        )
}
