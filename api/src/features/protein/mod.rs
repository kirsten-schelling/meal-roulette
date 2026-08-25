mod handlers;
mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/proteins",
            get(handlers::get_proteins).post(handlers::create_protein),
        )
        .route(
            "/proteins/{id}",
            get(handlers::get_protein)
                .put(handlers::update_protein)
                .delete(handlers::delete_protein),
        )
}
