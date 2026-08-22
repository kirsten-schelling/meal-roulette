use axum::Router;

use crate::features::recipe;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new().merge(recipe::routes()).with_state(state)
}
