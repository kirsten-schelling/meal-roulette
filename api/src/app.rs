use axum::Router;

use crate::features::{
    cook_log, cuisine, household, ingredient, meal_plan, planned_meal, protein, recipe,
    recipe_ingredient, recipe_tag, shopping_list, shopping_list_item, slept_ingredient, tag, unit,
};
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .merge(recipe::routes())
        .merge(ingredient::routes())
        .merge(unit::routes())
        .merge(recipe_ingredient::routes())
        .merge(household::routes())
        .merge(protein::routes())
        .merge(cuisine::routes())
        .merge(tag::routes())
        .merge(recipe_tag::routes())
        .merge(meal_plan::routes())
        .merge(planned_meal::routes())
        .merge(cook_log::routes())
        .merge(shopping_list::routes())
        .merge(shopping_list_item::routes())
        .merge(slept_ingredient::routes())
        .with_state(state)
}
