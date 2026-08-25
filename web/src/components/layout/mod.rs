//! App shell — the persistent frame around every routed page. `AppLayout`
//! composes `Header` (brand + global actions) and `Sidebar` (section nav)
//! around a routed `<Outlet/>`. Styling lives in `assets/primitives.css`
//! under the "APP SHELL" section.

pub mod app_layout;
pub mod header;
pub mod sidebar;

pub use app_layout::AppLayout;
pub use header::Header;
pub use sidebar::{NavEntry, Sidebar};
