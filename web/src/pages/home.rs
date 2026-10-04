//! Landing dashboard: this week's dinners from the latest rolled plan, plus
//! shortcuts into the app.

use crate::components::ui::*;
use crate::features::plan::hooks::use_plan::use_get_latest_plan;
use crate::models::day_label;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let plan = use_get_latest_plan();

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
                    match plan() {
                        None => rsx! {
                            div { class: "mr-tablestate", Spinner {} }
                        },
                        Some(Err(e)) => rsx! {
                            Alert { tone: Tone::Red, title: "Couldn't load this week", "{e}" }
                        },
                        Some(Ok(None)) => rsx! {
                            div { class: "mr-empty",
                                div { class: "mr-empty__icon", aria_hidden: "true", "🎲" }
                                Heading { level: 3, "No week rolled yet" }
                                Text { muted: true, "Spin the roller to fill the week." }
                                Link { to: Route::Roll {}, class: "mr-btn mr-btn--solid mr-btn--lg", "Roll a week" }
                            }
                        },
                        Some(Ok(Some(detail))) => rsx! {
                            div { class: "mr-week",
                                for m in detail.meals.iter() {
                                    div { key: "{m.id}", class: "mr-week__row",
                                        span { class: "mr-week__day", "{day_label(m.scheduled_date)}" }
                                        span { class: "mr-cell-strong", "{m.title}" }
                                    }
                                }
                            }
                            Link { to: Route::PlannedMeals {}, class: "mr-btn mr-btn--outline mr-btn--md", "Open planned meals" }
                        },
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
        Link { to, class: "mr-card mr-dashlink",
            div {
                class: "mr-card__body",
                style: "display:flex; align-items:center; gap:.7rem;",
                span {
                    style: "font-size:1.5rem; line-height:1;",
                    aria_hidden: "true",
                    "{icon}"
                }
                div {
                    div { style: "font-weight:600;", "{label}" }
                    div { style: "color:var(--mr-text-muted); font-size:.85rem;", "{hint}" }
                }
                span { class: "mr-dashlink__arrow", aria_hidden: "true", "›" }
            }
        }
    }
}
