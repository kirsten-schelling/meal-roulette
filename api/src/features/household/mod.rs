mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/households",
            get(handlers::get_households).post(handlers::create_household),
        )
        .route(
            "/households/{id}",
            get(handlers::get_household)
                .put(handlers::update_household)
                .delete(handlers::delete_household),
        )
}
