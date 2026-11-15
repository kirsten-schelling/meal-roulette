//! What the household estimates it has in stock. Counts rise when a shop is cleared from
//! the shopping list and fall when a dinner is cooked; the shopping list subtracts them.
//! They are estimates, so every count can be corrected here.

use crate::components::ui::{Alert, Card, Heading, Icon, IconName, Spinner, Text, Tone};
use crate::features::pantry::components::{AddToPantry, PantryRow};
use crate::features::pantry::hooks::use_pantry::use_get_pantry;
use crate::models::error_message;
use dioxus::prelude::*;

#[component]
pub fn Pantry() -> Element {
    let mut pantry = use_get_pantry();

    rsx! {
        div { class: "mr-page",
            header { class: "mr-page__head",
                Heading { level: 1, "Pantry" }
                Text { muted: true,
                    "An estimate of what you have. The shopping list leaves out what's covered here, so fix a count when it drifts."
                }
            }

            div { class: "mr-cooklog",
                AddToPantry { on_saved: move |_| pantry.restart() }

                match pantry() {
                    None => rsx! {
                        div { class: "mr-tablestate", Spinner {} }
                    },
                    Some(Err(e)) => rsx! {
                        Alert { tone: Tone::Red, title: "Couldn't load the pantry", "{error_message(&e)}" }
                    },
                    Some(Ok(lines)) if lines.is_empty() => rsx! {
                        div { class: "mr-empty",
                            div { class: "mr-empty__icon", aria_hidden: "true", Icon { name: IconName::Archive } }
                            Heading { level: 3, "Nothing counted yet" }
                            Text { muted: true,
                                "Clear a shop on the shopping list to stock the pantry, or set a count above."
                            }
                        }
                    },
                    Some(Ok(lines)) => rsx! {
                        Card {
                            for line in lines {
                                PantryRow {
                                    key: "{line.ingredient_id}-{line.qty}",
                                    line,
                                    on_saved: move |_| pantry.restart(),
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
