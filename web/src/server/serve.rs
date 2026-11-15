use api::state::AppState;
use dioxus::prelude::*;
use dioxus::server::axum::http::{header::CACHE_CONTROL, HeaderValue};
use dioxus::server::axum::response::Response;
use dioxus::server::axum::{middleware, Extension, Router};

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
            .layer(Extension(state))
            .layer(middleware::map_response(revalidate_by_default));
        Ok(router)
    })
}

/// dx marks its content-hashed bundle files immutable. Everything else, index.html above all,
/// is revalidated on each load so a reload always finds the current bundle's file names.
async fn revalidate_by_default(mut response: Response) -> Response {
    response
        .headers_mut()
        .entry(CACHE_CONTROL)
        .or_insert(HeaderValue::from_static("no-cache"));
    response
}
