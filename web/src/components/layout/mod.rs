//! App shell — the persistent frame around every routed page. `AppLayout`
//! composes `Sidebar` (brand, section nav and theme control) on wide screens
//! and `MobileNav` (top bar plus a slide-out drawer) on narrow ones around a
//! routed `<Outlet/>`. Styling lives in `assets/primitives.css` under "APP SHELL".

pub mod app_layout;
pub mod mobile_nav;
pub mod sidebar;
pub mod theme_toggle;

pub use app_layout::AppLayout;
pub use mobile_nav::MobileNav;
pub use sidebar::{NavEntry, Sidebar};
pub use theme_toggle::ThemeToggle;
