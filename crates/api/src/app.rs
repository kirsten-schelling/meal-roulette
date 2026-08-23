use axum::Router;

use crate::features::{ingredient, recipe, recipe_ingredient, unit};
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .merge(recipe::routes())
        .merge(ingredient::routes())
        .merge(unit::routes())
        .merge(recipe_ingredient::routes())
        .with_state(state)
}
