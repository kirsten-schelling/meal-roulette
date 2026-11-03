use crate::server::api::plan::{get_latest_plan, get_plan};
use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;
use uuid::Uuid;

pub use crate::server::api::plan::{reroll_planned_meal, set_plan_favourite, swap_planned_meal};

/// One plan by id, or the latest active plan when `id` is `None`. `Ok(None)` when every plan is
/// archived or none was rolled.
pub fn use_get_plan(
    id: ReadSignal<Option<Uuid>>,
) -> Resource<Result<Option<MealPlanDetail>, ServerFnError>> {
    use_resource(move || {
        let id = id();
        async move {
            match id {
                Some(id) => get_plan(id).await.map(Some),
                None => get_latest_plan().await,
            }
        }
    })
}
