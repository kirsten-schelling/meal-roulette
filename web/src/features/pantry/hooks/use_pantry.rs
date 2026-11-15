use crate::server::api::pantry::get_pantry;
use dioxus::prelude::*;
use domain::pantry::PantryLine;

pub use crate::server::api::pantry::set_pantry_qty;

pub fn use_get_pantry() -> Resource<Result<Vec<PantryLine>, ServerFnError>> {
    use_resource(move || async move { get_pantry().await })
}
