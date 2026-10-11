mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tags", get(handlers::get_tags).post(handlers::create_tag))
        .route(
            "/tags/{id}",
            get(handlers::get_tag)
                .put(handlers::update_tag)
                .delete(handlers::delete_tag),
        )
}
