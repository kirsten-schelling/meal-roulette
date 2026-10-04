use crate::server::api::roll_preset::get_roll_presets;
use dioxus::prelude::*;
use domain::roll_preset::RollPreset;

pub use crate::server::api::roll_preset::{
    add_roll_preset, delete_roll_preset, set_default_roll_preset, update_roll_preset,
};

pub fn use_get_roll_presets() -> Resource<Result<Vec<RollPreset>, ServerFnError>> {
    use_resource(move || async move { get_roll_presets().await })
}
