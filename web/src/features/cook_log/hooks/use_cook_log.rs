use crate::server::api::cook_log::{get_cook_log, get_planned_meal_cook};
use dioxus::prelude::*;
use domain::cook_log::{CookLog, CookLogEntry};
use uuid::Uuid;

pub use crate::server::api::cook_log::{delete_cook, log_cook};

pub fn use_get_cook_log() -> Resource<Result<Vec<CookLogEntry>, ServerFnError>> {
    use_resource(move || async move { get_cook_log().await })
}

/// The cook logged for a planned dinner, if any. Refetches when the dinner changes.
pub fn use_get_planned_meal_cook(
    planned_meal_id: Uuid,
) -> Resource<Result<Option<CookLog>, ServerFnError>> {
    use_resource(use_reactive(&planned_meal_id, |id| async move {
        get_planned_meal_cook(id).await
    }))
}
