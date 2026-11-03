//! The solver-backed operations: roll a plan (or preview one without saving it),
//! re-roll one of its days. The
//! paths follow the resources they act on; the handlers live here rather than
//! under `meal_plan` / `planned_meal` because both need the solver.

mod handlers;
pub mod service;

use crate::state::AppState;
use axum::{routing::post, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/roll", post(handlers::roll))
        .route("/roll/preview", post(handlers::preview))
        .route("/roll/preview/reroll", post(handlers::reroll_preview))
        .route("/roll/keep", post(handlers::keep))
        .route("/roll/fit", post(handlers::fit))
        .route("/planned-meals/{id}/reroll", post(handlers::reroll))
}
