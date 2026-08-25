//! The section navigation. `Sidebar` is deliberately dumb: it renders whatever
//! list of `NavEntry` it's handed. The links themselves come from `AppLayout`.
//!
//! `Link` handles the routing and adds `mr-nav__link--active` to whichever entry
//! matches the current URL, so the active-page highlight is free.

use crate::router::Route;
use dioxus::prelude::*;

/// One row in the sidebar: an icon, a label, and where it navigates.
#[derive(Clone, PartialEq)]
pub struct NavEntry {
    pub icon: &'static str,
    pub label: &'static str,
    pub to: Route,
}

#[component]
pub fn Sidebar(entries: Vec<NavEntry>) -> Element {
    rsx! {
        aside { class: "mr-sidebar",
            nav { class: "mr-nav", aria_label: "Sections",
                for entry in entries {
                    Link {
                        key: "{entry.label}",
                        class: "mr-nav__link",
                        active_class: "mr-nav__link--active".to_string(),
                        to: entry.to.clone(),
                        span { class: "mr-nav__icon", aria_hidden: "true", "{entry.icon}" }
                        span { class: "mr-nav__label", "{entry.label}" }
                    }
                }
            }
        }
    }
}
