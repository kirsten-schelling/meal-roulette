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
            "/cuisines",
            get(handlers::get_cuisines).post(handlers::create_cuisine),
        )
        .route(
            "/cuisines/{id}",
            get(handlers::get_cuisine)
                .put(handlers::update_cuisine)
                .delete(handlers::delete_cuisine),
        )
        .route(
            "/cuisines/{id}/references",
            get(handlers::get_cuisine_references),
        )
        .route(
            "/cuisines/{id}/cascade",
            delete(handlers::delete_cuisine_cascade),
        )
}
