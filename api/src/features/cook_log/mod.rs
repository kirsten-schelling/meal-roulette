mod handlers;
mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/cook-logs",
            get(handlers::get_cook_logs).post(handlers::create_cook_log),
        )
        .route(
            "/cook-logs/{id}",
            get(handlers::get_cook_log)
                .put(handlers::update_cook_log)
                .delete(handlers::delete_cook_log),
        )
}
