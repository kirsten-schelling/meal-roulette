use api::error::AppError;
use dioxus::prelude::ServerFnError;

/// The one place a data-layer error becomes a `ServerFnError`, carrying the same status code
/// the REST routes answer with.
pub fn server_fn_error(error: impl Into<AppError>) -> ServerFnError {
    let error = error.into();
    let status = error.status();
    eprintln!("[web] {status}: {error}");
    ServerFnError::ServerError {
        message: error.to_string(),
        code: status.as_u16(),
        details: None,
    }
}
