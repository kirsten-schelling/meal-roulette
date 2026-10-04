//! Landing dashboard: a snapshot of the week plus shortcuts into the app. The
//! week panel is a placeholder until the planned-meals data is wired in.

use crate::components::ui::*;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "mr-page",
            header { class: "mr-page__head",
                Heading { level: 1, "What's for dinner?" }
                Text { muted: true,
                    "Your week at a glance. Roll a plan, then turn it into a shopping list."
                }
            }

            VStack { gap: 1.25,
                Card { header: "This week".to_string(),
                    div { class: "mr-empty",
                        div { class: "mr-empty__icon", aria_hidden: "true", "🎲" }
                        Heading { level: 3, "No week rolled yet" }
                        Text { muted: true,
                            "Spin the roller to fill the week. Your meals show up here once the plan is wired in."
                        }
                        Link {
                            to: Route::Roll {},
                            class: "mr-btn mr-btn--solid mr-btn--lg",
                            "Roll a week"
                        }
                    }
                }

                div { style: "display:grid; grid-template-columns:repeat(auto-fit, minmax(12rem, 1fr)); gap:.9rem;",
                    DashLink {
                        to: Route::Recipes {},
                        icon: "📖",
                        label: "Recipes",
                        hint: "Browse and add recipes",
                    }
                    DashLink {
                        to: Route::PlannedMeals {},
                        icon: "📅",
                        label: "Planned Meals",
                        hint: "The week the roller picked",
                    }
                    DashLink {
                        to: Route::ShoppingList {},
                        icon: "🛒",
                        label: "Shopping List",
                        hint: "What to buy this week",
                    }
                    DashLink {
                        to: Route::Library {},
                        icon: "🧺",
                        label: "Library",
                        hint: "Ingredients, proteins, cuisines",
                    }
                }
            }
        }
    }
}

/// A card that navigates to a section. Renders as a link so the whole card is
/// clickable and keyboard-focusable.
#[component]
fn DashLink(to: Route, icon: String, label: String, hint: String) -> Element {
    rsx! {
        Link { to: to, class: "mr-card mr-dashlink",
            div {
                class: "mr-card__body",
                style: "display:flex; align-items:center; gap:.7rem;",
                span { style: "font-size:1.5rem; line-height:1;", aria_hidden: "true", "{icon}" }
                div {
                    div { style: "font-weight:600;", "{label}" }
                    div { style: "color:var(--mr-text-muted); font-size:.85rem;", "{hint}" }
                }
                span { class: "mr-dashlink__arrow", aria_hidden: "true", "›" }
            }
        }
    }
}
