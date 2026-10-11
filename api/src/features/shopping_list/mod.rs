mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/shopping-lists",
            get(handlers::get_shopping_lists).post(handlers::create_shopping_list),
        )
        .route(
            "/shopping-lists/{id}",
            get(handlers::get_shopping_list)
                .put(handlers::update_shopping_list)
                .delete(handlers::delete_shopping_list),
        )
        .route(
            "/meal-plans/{id}/shopping-list",
            get(handlers::get_plan_shopping_list).post(handlers::generate_plan_shopping_list),
        )
}
