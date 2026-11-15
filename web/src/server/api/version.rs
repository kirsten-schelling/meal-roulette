use dioxus::prelude::*;

/// Stamped by the Dockerfile so the browser bundle and the server binary of one deploy agree.
/// Local builds fall back to "dev", which matches on both sides.
pub const BUILD_ID: &str = match option_env!("MR_BUILD_ID") {
    Some(id) => id,
    None => "dev",
};

#[server]
pub async fn get_build_id() -> Result<String, ServerFnError> {
    Ok(BUILD_ID.to_string())
}
