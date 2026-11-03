mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/units",
            get(handlers::get_units).post(handlers::create_unit),
        )
        .route(
            "/units/{id}",
            get(handlers::get_unit)
                .put(handlers::update_unit)
                .delete(handlers::delete_unit),
        )
        .route("/units/{id}/references", get(handlers::get_unit_references))
}
