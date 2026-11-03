mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{
    routing::{delete, get},
    Router,
};

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
        .route(
            "/proteins/{id}/references",
            get(handlers::get_protein_references),
        )
        .route(
            "/proteins/{id}/cascade",
            delete(handlers::delete_protein_cascade),
        )
}
