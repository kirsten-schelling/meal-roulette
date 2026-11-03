use crate::server::api::roll::count_fitting_recipes;
use dioxus::prelude::*;
use domain::roll::{FitCount, FitRequest};

/// How many recipes the roll can draw on under `request`, refetched whenever
/// the request changes.
pub fn use_fit_count(
    request: Memo<Option<FitRequest>>,
) -> Resource<Option<Result<FitCount, ServerFnError>>> {
    use_resource(move || async move {
        match request() {
            Some(request) => Some(count_fitting_recipes(request).await),
            None => None,
        }
    })
}
