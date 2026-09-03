//! The shell every page renders inside: a CSS-grid frame with the header
//! across the top, the sidebar down the left, and the routed page in the
//! main column. Owns only UI state (the light/dark toggle) — no app logic.

use crate::components::layout::{Header, NavEntry, Sidebar};
use crate::components::toast_master::ToastMaster;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn AppLayout() -> Element {
    // Light/dark is purely presentational, so the shell owns it. The token
    // stylesheet swaps palettes off the `data-theme` attribute below.
    let mut dark = use_signal(|| false);
    let theme = if dark() { "dark" } else { "light" };

    rsx! {
        div { class: "mr-scope mr-shell", "data-theme": "{theme}",
            // Inside the themed shell so the toast stack follows the toggle,
            // not the OS colour scheme.
            ToastMaster {
                Header { dark: dark(), on_toggle: move |v| dark.set(v) }
                Sidebar { entries: nav_entries() }
                main { class: "mr-main",
                    div { class: "mr-main__inner",
                        // The current route's page renders here.
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}

/// The sidebar's links. Hard-coded for the MVP — later this can come from
/// config or be filtered per user/household.
fn nav_entries() -> Vec<NavEntry> {
    vec![
        NavEntry {
            icon: "📖",
            label: "Recipes",
            to: Route::Recipes {},
        },
        NavEntry {
            icon: "🧺",
            label: "Library",
            to: Route::Library {},
        },
        NavEntry {
            icon: "🛒",
            label: "Shopping List",
            to: Route::ShoppingList {},
        },
        NavEntry {
            icon: "📓",
            label: "Cooking Log",
            to: Route::CookingLog {},
        },
        NavEntry {
            icon: "📅",
            label: "Planned Meals",
            to: Route::PlannedMeals {},
        },
    ]
}
