//! App shell — the persistent frame around every routed page. `AppLayout`
//! composes `Sidebar` (brand, section nav and theme control) on wide screens
//! and `TabBar` (bottom tabs plus a "More" sheet) on narrow ones around a
//! routed `<Outlet/>`. Styling lives in `assets/primitives.css` under "APP SHELL".

pub mod app_layout;
pub mod sidebar;
pub mod tab_bar;
pub mod theme_toggle;

pub use app_layout::AppLayout;
pub use sidebar::{NavEntry, Sidebar};
pub use tab_bar::TabBar;
pub use theme_toggle::ThemeToggle;
