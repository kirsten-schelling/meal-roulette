mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/shopping-list-items",
            get(handlers::get_shopping_list_items).post(handlers::create_shopping_list_item),
        )
        .route(
            "/shopping-list-items/{id}",
            get(handlers::get_shopping_list_item)
                .put(handlers::update_shopping_list_item)
                .delete(handlers::delete_shopping_list_item),
        )
        .route(
            "/shopping-lists/{id}/custom-items",
            post(handlers::add_custom_item),
        )
        .route(
            "/shopping-lists/{id}/custom-items/{item_id}",
            delete(handlers::remove_custom_item),
        )
}
