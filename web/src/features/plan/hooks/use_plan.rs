use crate::server::api::plan::get_latest_plan;
use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;

pub use crate::server::api::plan::{reroll_planned_meal, swap_planned_meal};

pub fn use_get_latest_plan() -> Resource<Result<Option<MealPlanDetail>, ServerFnError>> {
    use_resource(move || async move { get_latest_plan().await })
}
