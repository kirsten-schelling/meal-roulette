mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/planned-meals",
            get(handlers::get_planned_meals).post(handlers::create_planned_meal),
        )
        .route(
            "/planned-meals/{id}",
            get(handlers::get_planned_meal)
                .put(handlers::update_planned_meal)
                .delete(handlers::delete_planned_meal),
        )
}
