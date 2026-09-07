//! Server-side code for the web app.
pub mod api;

#[cfg(feature = "server")]
mod error;
#[cfg(feature = "server")]
mod serve;
#[cfg(feature = "server")]
mod state;

#[cfg(feature = "server")]
pub use {error::server_fn_error, serve::serve, state::app_state};
