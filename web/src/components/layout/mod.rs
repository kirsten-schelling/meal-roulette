//! App shell — the persistent frame around every routed page. `AppLayout`
//! composes `Sidebar` (brand, section nav and theme control) around a routed
//! `<Outlet/>`. Styling lives in `assets/primitives.css` under "APP SHELL".

pub mod app_layout;
pub mod sidebar;
pub mod theme_toggle;

pub use app_layout::AppLayout;
pub use sidebar::{NavEntry, Sidebar};
pub use theme_toggle::ThemeToggle;
