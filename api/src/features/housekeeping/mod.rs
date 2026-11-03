mod handlers;
pub mod service;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/housekeeping", get(handlers::get_housekeeping))
}
