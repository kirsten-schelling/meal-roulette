//! The one persistent sidebar: brand on top, section navigation in the middle,
//! palette and theme controls at the foot. `Sidebar` is deliberately dumb: it renders
//! whatever lists of `NavEntry` it's handed, the second under a quieter
//! "Manage" heading. The links themselves come from `AppLayout`.
//!
//! `Link` handles the routing and adds `mr-nav__link--active` to whichever entry
//! matches the current URL, so the active-page highlight is free.

use crate::components::layout::{PalettePicker, ThemeToggle, VersionLine};
use crate::components::ui::{BrandMark, Icon, IconName};
use crate::router::Route;
use dioxus::prelude::*;

/// One row in the sidebar: an icon, a label, and where it navigates.
#[derive(Clone, PartialEq)]
pub struct NavEntry {
    pub icon: IconName,
    pub label: &'static str,
    pub to: Route,
}

#[component]
pub fn Sidebar(primary: Vec<NavEntry>, manage: Vec<NavEntry>) -> Element {
    rsx! {
        aside { class: "mr-sidebar",
            Link { class: "mr-brand", to: Route::Home {},
                BrandMark {}
                span { class: "mr-brand__name", "Meal Roulette" }
            }
            nav { class: "mr-nav", aria_label: "Sections",
                NavLinks { entries: primary }
                div { class: "mr-nav__group", id: "mr-nav-manage", "Manage" }
                NavLinks { entries: manage, quiet: true }
            }
            div { class: "mr-sidebar__foot",
                PalettePicker {}
                ThemeToggle {}
                VersionLine {}
            }
        }
    }
}

#[component]
pub fn NavLinks(
    entries: Vec<NavEntry>,
    #[props(default)] quiet: bool,
    #[props(optional)] on_navigate: Option<EventHandler<()>>,
) -> Element {
    let class = if quiet {
        "mr-nav__link mr-nav__link--quiet"
    } else {
        "mr-nav__link"
    };
    rsx! {
        for entry in entries {
            Link {
                key: "{entry.label}",
                class,
                active_class: "mr-nav__link--active".to_string(),
                to: entry.to.clone(),
                onclick: move |_| {
                    if let Some(h) = on_navigate {
                        h.call(());
                    }
                },
                span { class: "mr-nav__icon", Icon { name: entry.icon } }
                span { class: "mr-nav__label", "{entry.label}" }
            }
        }
    }
}
