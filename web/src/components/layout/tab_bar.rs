//! The phone navigation: a bar pinned to the bottom edge with the four
//! everyday destinations and a "More" tab that opens a bottom sheet holding the
//! Manage links and the theme control. Hidden above 720px, where the sidebar
//! shows instead.

use crate::components::layout::{NavEntry, ThemeToggle};
use crate::components::ui::{Icon, IconName, Modal};
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn TabBar(manage: Vec<NavEntry>) -> Element {
    let mut more_open = use_signal(|| false);
    let route = use_route::<Route>();
    let in_manage = manage.iter().any(|entry| entry.to == route);

    rsx! {
        nav { class: "mr-tabbar", aria_label: "Main",
            TabLink { to: Route::Home {}, icon: IconName::CalendarWeek, label: "This week" }
            TabLink { to: Route::Roll {}, icon: IconName::Dice, label: "Roll" }
            TabLink { to: Route::ShoppingList {}, icon: IconName::Checklist, label: "List" }
            TabLink { to: Route::Recipes {}, icon: IconName::ChefHat, label: "Recipes" }
            button {
                r#type: "button",
                class: if in_manage { "mr-tab mr-tab--active" } else { "mr-tab" },
                aria_haspopup: "dialog",
                onclick: move |_| more_open.set(true),
                span { class: "mr-tab__icon", Icon { name: IconName::Dots } }
                span { class: "mr-tab__label", "More" }
            }
        }
        if more_open() {
            Modal { title: "More".to_string(), on_close: move |_| more_open.set(false),
                div { class: "mr-sheetnav",
                    for entry in manage {
                        Link {
                            key: "{entry.label}",
                            class: "mr-nav__link",
                            active_class: "mr-nav__link--active".to_string(),
                            to: entry.to.clone(),
                            onclick: move |_| more_open.set(false),
                            span { class: "mr-nav__icon", Icon { name: entry.icon } }
                            span { class: "mr-nav__label", "{entry.label}" }
                        }
                    }
                    div { class: "mr-sheetnav__theme", ThemeToggle {} }
                }
            }
        }
    }
}

#[component]
fn TabLink(to: Route, icon: IconName, label: &'static str) -> Element {
    rsx! {
        Link {
            class: "mr-tab",
            active_class: "mr-tab--active".to_string(),
            to,
            span { class: "mr-tab__icon", Icon { name: icon } }
            span { class: "mr-tab__label", "{label}" }
        }
    }
}
