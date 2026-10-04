mod handlers;
pub mod repository;

use crate::state::AppState;
use axum::{
    routing::{get, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/roll-presets",
            get(handlers::get_roll_presets).post(handlers::create_roll_preset),
        )
        .route(
            "/roll-presets/{id}",
            get(handlers::get_roll_preset)
                .put(handlers::update_roll_preset)
                .delete(handlers::delete_roll_preset),
        )
        .route(
            "/roll-presets/{id}/default",
            put(handlers::set_default_roll_preset),
        )
}
