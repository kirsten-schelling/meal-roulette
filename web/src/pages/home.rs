//! Landing dashboard: the dinner scheduled for today in the latest rolled plan, or a
//! prompt to put the plan on days, plus shortcuts into the app.

use crate::components::ui::*;
use crate::features::housekeeping::components::HousekeepingCard;
use crate::features::plan::hooks::use_plan::use_get_latest_plan;
use crate::hooks::use_today::use_today;
use crate::models::error_message;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let plan = use_get_latest_plan();
    let today = use_today();

    rsx! {
        div { class: "mr-page",
            header { class: "mr-page__head",
                Heading { level: 1, "What's for dinner?" }
                Text { muted: true,
                    "Tonight's dinner at a glance. Roll a plan, put it on days, then turn it into a "
                    Link { to: Route::ShoppingList {}, class: "mr-textlink", "shopping list" }
                    "."
                }
            }

            VStack { gap: 1.25,
                Card { header: "Today".to_string(),
                    match plan() {
                        None => rsx! {
                            div { class: "mr-tablestate", Spinner {} }
                        },
                        Some(Err(e)) => rsx! {
                            Alert { tone: Tone::Red, title: "Couldn't load today's dinner", "{error_message(&e)}" }
                        },
                        Some(Ok(None)) => rsx! {
                            div { class: "mr-empty",
                                div { class: "mr-empty__icon", Icon { name: IconName::Dice } }
                                Heading { level: 3, "No week rolled yet" }
                                Text { muted: true, "Spin the roller to fill the week." }
                                Link { to: Route::Roll {}, class: "mr-btn mr-btn--solid mr-btn--lg", "Roll a week" }
                            }
                        },
                        Some(Ok(Some(detail))) => rsx! {
                            {
                                let tonight = today()
                                    .and_then(|date| {
                                        detail.meals.iter().find(|m| m.scheduled_date == Some(date))
                                    });
                                match (tonight, today()) {
                                    (Some(meal), _) => rsx! {
                                        div { class: "mr-week",
                                            div { class: "mr-week__row",
                                                span { class: "mr-week__day", "Tonight" }
                                                span { class: "mr-cell-strong", "{meal.title}" }
                                            }
                                        }
                                    },
                                    (None, Some(_)) => rsx! {
                                        Text { muted: true, "Nothing is scheduled for today. Put the week on days to see tonight's dinner here." }
                                    },
                                    (None, None) => rsx! {
                                        div { class: "mr-tablestate", Spinner {} }
                                    },
                                }
                            }
                            div { class: "mr-week__actions",
                                Link {
                                    to: Route::PlannedMeals {},
                                    class: "mr-btn mr-btn--outline mr-btn--md",
                                    if detail.meals.iter().any(|m| m.scheduled_date.is_none()) {
                                        "Put the week on days"
                                    } else {
                                        "Open planned meals"
                                    }
                                }
                                Link {
                                    to: Route::PlanShoppingList { plan_id: detail.plan.id },
                                    class: "mr-btn mr-btn--outline mr-btn--md",
                                    Icon { name: IconName::Checklist }
                                    "Shopping list"
                                }
                            }
                        },
                    }
                }

                HousekeepingCard {}

                div { style: "display:grid; grid-template-columns:repeat(auto-fit, minmax(12rem, 1fr)); gap:.9rem;",
                    DashLink {
                        to: Route::Recipes {},
                        icon: IconName::ChefHat,
                        label: "Recipes",
                        hint: "Browse and add recipes",
                    }
                    DashLink {
                        to: Route::PlannedMeals {},
                        icon: IconName::CalendarWeek,
                        label: "Planned Meals",
                        hint: "The week the roller picked",
                    }
                    DashLink {
                        to: Route::ShoppingList {},
                        icon: IconName::Checklist,
                        label: "Shopping List",
                        hint: "What to buy for this week's dinners",
                    }
                    DashLink {
                        to: Route::History {},
                        icon: IconName::Archive,
                        label: "History",
                        hint: "Past weeks, favourites, archive",
                    }
                    DashLink {
                        to: Route::Library { manage: String::new() },
                        icon: IconName::Carrot,
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
fn DashLink(to: Route, icon: IconName, label: String, hint: String) -> Element {
    rsx! {
        Link { to, class: "mr-card mr-dashlink",
            div {
                class: "mr-card__body",
                style: "display:flex; align-items:center; gap:.7rem;",
                span { style: "font-size:1.5rem; line-height:1; color:var(--mr-accent);",
                    Icon { name: icon }
                }
                div {
                    div { style: "font-weight:600;", "{label}" }
                    div { style: "color:var(--mr-text-muted); font-size:.85rem;", "{hint}" }
                }
                span { class: "mr-dashlink__arrow", Icon { name: IconName::ChevronRight } }
            }
        }
    }
}
