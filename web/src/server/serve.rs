use api::state::AppState;
use dioxus::prelude::*;
use dioxus::server::axum::{Extension, Router};

/// One process: the Dioxus app plus the REST routes nested under `/api`. The `AppState` is
/// attached to every request as an extension so `#[server]` functions can read it back
/// through `app_state`.
pub fn serve(app: fn() -> Element) -> ! {
    dioxus::serve(move || async move {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let state = AppState::connect(&database_url).await?;
        let router = Router::new()
            .serve_dioxus_application(ServeConfig::new(), app)
            .nest("/api", api::app::build(state.clone()))
            .layer(Extension(state));
        Ok(router)
    })
}
