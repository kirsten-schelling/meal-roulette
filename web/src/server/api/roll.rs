use dioxus::prelude::*;
use domain::meal_plan::MealPlanDetail;
use domain::roll::{
    FitCount, FitRequest, KeepRollRequest, RerollPreview, RerollPreviewRequest, RollPreview,
    RollRequest,
};

#[server]
pub async fn preview_roll(request: RollRequest) -> Result<RollPreview, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll::service;

    let state = app_state()?;
    service::preview_roll(&state.db, request)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn reroll_preview(request: RerollPreviewRequest) -> Result<RerollPreview, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll::service;

    let state = app_state()?;
    service::reroll_preview(&state.db, request)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn keep_roll(request: KeepRollRequest) -> Result<MealPlanDetail, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll::service;

    let state = app_state()?;
    service::keep_roll(&state.db, request)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn count_fitting_recipes(request: FitRequest) -> Result<FitCount, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::roll::service;

    let state = app_state()?;
    service::fit_count(&state.db, request)
        .await
        .map_err(server_fn_error)
}
