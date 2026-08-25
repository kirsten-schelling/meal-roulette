mod handlers;
mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

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
}
