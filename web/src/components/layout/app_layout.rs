//! The shell every page renders inside: a CSS-grid frame with the sidebar down
//! the left and the routed page in the main column. No state of its own.

use crate::components::layout::{NavEntry, Sidebar};
use crate::components::toast_master::ToastMaster;
use crate::components::ui::IconName;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        div { class: "mr-scope mr-shell",
            ToastMaster {
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
            icon: IconName::Dice,
            label: "Roll",
            to: Route::Roll {},
        },
        NavEntry {
            icon: IconName::ChefHat,
            label: "Recipes",
            to: Route::Recipes {},
        },
        NavEntry {
            icon: IconName::Carrot,
            label: "Library",
            to: Route::Library {
                manage: String::new(),
            },
        },
        NavEntry {
            icon: IconName::Checklist,
            label: "Shopping List",
            to: Route::ShoppingList {},
        },
        NavEntry {
            icon: IconName::History,
            label: "Cooking Log",
            to: Route::CookingLog {},
        },
        NavEntry {
            icon: IconName::CalendarWeek,
            label: "Planned Meals",
            to: Route::PlannedMeals {},
        },
        NavEntry {
            icon: IconName::Archive,
            label: "History",
            to: Route::History {},
        },
    ]
}
