//! The shell every page renders inside: a CSS-grid frame with the sidebar down
//! the left and the routed page in the main column. Below 720px the sidebar is
//! hidden and `TabBar` takes over along the bottom edge. No state of its own.

use crate::components::layout::{NavEntry, Sidebar, TabBar};
use crate::components::toast_master::ToastMaster;
use crate::components::ui::IconName;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        div { class: "mr-scope mr-shell",
            ToastMaster {
                Sidebar { primary: primary_entries(), manage: manage_entries() }
                main { class: "mr-main",
                    div { class: "mr-main__inner",
                        // The current route's page renders here.
                        Outlet::<Route> {}
                    }
                }
                TabBar { manage: manage_entries() }
            }
        }
    }
}

/// The everyday loop, in the order it is used: look at the week, roll a new
/// one, shop for it, browse recipes. Cooking Log stays out until it exists.
fn primary_entries() -> Vec<NavEntry> {
    vec![
        NavEntry {
            icon: IconName::CalendarWeek,
            label: "This week",
            to: Route::Home {},
        },
        NavEntry {
            icon: IconName::Dice,
            label: "Roll",
            to: Route::Roll {},
        },
        NavEntry {
            icon: IconName::Checklist,
            label: "Shopping list",
            to: Route::ShoppingList {},
        },
        NavEntry {
            icon: IconName::ChefHat,
            label: "Recipes",
            to: Route::Recipes {},
        },
    ]
}

/// Upkeep rather than daily use: shown quieter, and behind "More" on a phone.
fn manage_entries() -> Vec<NavEntry> {
    vec![
        NavEntry {
            icon: IconName::Carrot,
            label: "Library",
            to: Route::Library {},
        },
        NavEntry {
            icon: IconName::Archive,
            label: "History",
            to: Route::History {},
        },
    ]
}
