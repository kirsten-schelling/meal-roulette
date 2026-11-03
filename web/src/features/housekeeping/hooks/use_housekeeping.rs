use crate::server::api::housekeeping::get_housekeeping;
use dioxus::prelude::*;
use domain::housekeeping::Finding;

pub fn use_get_housekeeping() -> Resource<Result<Vec<Finding>, ServerFnError>> {
    use_resource(move || async move {
        let findings: Result<Vec<Finding>, ServerFnError> = get_housekeeping().await;
        findings
    })
}
