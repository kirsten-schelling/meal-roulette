mod handlers;
mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/meal-plans",
            get(handlers::get_meal_plans).post(handlers::create_meal_plan),
        )
        .route(
            "/meal-plans/{id}",
            get(handlers::get_meal_plan)
                .put(handlers::update_meal_plan)
                .delete(handlers::delete_meal_plan),
        )
}
