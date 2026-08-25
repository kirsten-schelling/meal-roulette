mod handlers;
mod repository;

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
}
