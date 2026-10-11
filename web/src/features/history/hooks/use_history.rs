use crate::server::api::history::get_plan_history;
use dioxus::prelude::*;
use domain::meal_plan::MealPlanSummary;

pub use crate::server::api::plan::{delete_plan, set_plan_archived, set_plan_favourite};

pub fn use_get_plan_history() -> Resource<Result<Vec<MealPlanSummary>, ServerFnError>> {
    use_resource(move || async move { get_plan_history().await })
}
