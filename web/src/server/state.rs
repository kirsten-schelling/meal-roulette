use api::state::AppState;
use dioxus::fullstack::FullstackContext;
use dioxus::prelude::ServerFnError;

pub fn app_state() -> Result<AppState, ServerFnError> {
    FullstackContext::current()
        .and_then(|request| request.extension::<AppState>())
        .ok_or_else(|| ServerFnError::new("AppState is not attached to this request"))
}
