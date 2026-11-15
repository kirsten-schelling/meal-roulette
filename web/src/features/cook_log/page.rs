//! What was actually cooked, grouped by day, newest first, with a card to log a cook
//! that wasn't on a plan. A cook can be taken off the log if it was logged by mistake.

use std::collections::HashMap;

use crate::components::ui::{
    Alert, Card, ConfirmDialog, Heading, Icon, IconButton, IconName, Spinner, Text, Tone,
};
use crate::features::cook_log::components::LogCookCard;
use crate::features::cook_log::hooks::use_cook_log::{delete_cook, use_get_cook_log};
use crate::hooks::use_toast::use_toast;
use crate::models::{date_label, error_message};
use crate::router::Route;
use dioxus::prelude::*;
use domain::cook_log::CookLogEntry;
use time::Date;
use uuid::Uuid;

#[component]
pub fn CookingLog() -> Element {
    let mut log = use_get_cook_log();
    let toast = use_toast();
    // The cook waiting on the remove confirmation; `Some` mounts the dialog.
    let mut removing = use_signal(|| None as Option<CookLogEntry>);
    let mut remove_busy = use_signal(|| false);

    let mut remove_confirmed = move || {
        let Some(entry) = removing() else {
            return;
        };
        remove_busy.set(true);
        spawn(async move {
            match delete_cook(entry.id).await {
                Ok(_) => {
                    log.restart();
                    toast.show(
                        Some("Removed".to_string()),
                        format!("{} is off the cooking log.", entry.title),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't remove".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            removing.set(None);
            remove_busy.set(false);
        });
    };

    rsx! {
        div { class: "mr-page",
            header { class: "mr-page__head",
                Heading { level: 1, "Cooking Log" }
                Text { muted: true, "A record of what you've actually cooked." }
            }

            div { class: "mr-cooklog",
                LogCookCard { on_logged: move |_| log.restart() }

                match log() {
                    None => rsx! {
                        div { class: "mr-tablestate", Spinner {} }
                    },
                    Some(Err(e)) => rsx! {
                        Alert { tone: Tone::Red, title: "Couldn't load the cooking log", "{error_message(&e)}" }
                    },
                    Some(Ok(entries)) if entries.is_empty() => rsx! {
                        div { class: "mr-empty",
                            div { class: "mr-empty__icon", aria_hidden: "true", Icon { name: IconName::History } }
                            Heading { level: 3, "Nothing cooked yet" }
                            Text { muted: true,
                                "Tap Cooked it on tonight's dinner at home, or log a cook above."
                            }
                        }
                    },
                    Some(Ok(entries)) => {
                        let counts = cook_counts(&entries);
                        rsx! {
                            for (day, cooks) in cooks_by_day(entries) {
                                Card { key: "{day}", header: date_label(day),
                                    for entry in cooks {
                                        {
                                            let times = counts[&entry.recipe_id];
                                            let doomed = entry.clone();
                                            rsx! {
                                                div { key: "{entry.id}", class: "mr-cooklog__row",
                                                    div { class: "mr-cooklog__text",
                                                        Link {
                                                            to: Route::RecipePage { id: entry.recipe_id },
                                                            class: "mr-cell-strong",
                                                            "{entry.title}"
                                                        }
                                                        if times > 1 {
                                                            span { class: "mr-cooklog__meta", "Cooked {times} times" }
                                                        }
                                                    }
                                                    IconButton {
                                                        label: format!("Remove {} from the log", entry.title),
                                                        danger: true,
                                                        onclick: move |_| removing.set(Some(doomed.clone())),
                                                        Icon { name: IconName::X }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(entry) = removing() {
                ConfirmDialog {
                    title: "Remove this cook?".to_string(),
                    confirm_label: "Remove".to_string(),
                    danger: true,
                    loading: remove_busy(),
                    on_confirm: move |_| remove_confirmed(),
                    on_cancel: move |_| removing.set(None),
                    "{entry.title} on {date_label(entry.cooked_on)} comes off the cooking log."
                }
            }
        }
    }
}

/// The log's entries split into one group per day, keeping the log's newest-first order.
fn cooks_by_day(entries: Vec<CookLogEntry>) -> Vec<(Date, Vec<CookLogEntry>)> {
    let mut days: Vec<(Date, Vec<CookLogEntry>)> = Vec::new();
    for entry in entries {
        match days.last_mut() {
            Some((day, cooks)) if *day == entry.cooked_on => cooks.push(entry),
            _ => days.push((entry.cooked_on, vec![entry])),
        }
    }
    days
}

/// How many times each recipe appears in the log.
fn cook_counts(entries: &[CookLogEntry]) -> HashMap<Uuid, usize> {
    let mut counts = HashMap::new();
    for entry in entries {
        *counts.entry(entry.recipe_id).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn cook(recipe_id: Uuid, title: &str, cooked_on: Date) -> CookLogEntry {
        CookLogEntry {
            id: Uuid::now_v7(),
            recipe_id,
            title: title.to_string(),
            cooked_on,
        }
    }

    #[test]
    fn cooks_on_the_same_day_share_a_group_in_log_order() {
        let pasta = Uuid::now_v7();
        let curry = Uuid::now_v7();
        let entries = vec![
            cook(pasta, "Pasta", date!(2026 - 11 - 14)),
            cook(curry, "Curry", date!(2026 - 11 - 14)),
            cook(pasta, "Pasta", date!(2026 - 11 - 9)),
        ];

        let grouped: Vec<(Date, Vec<String>)> = cooks_by_day(entries)
            .into_iter()
            .map(|(day, cooks)| (day, cooks.into_iter().map(|c| c.title).collect()))
            .collect();
        assert_eq!(
            grouped,
            vec![
                (
                    date!(2026 - 11 - 14),
                    vec!["Pasta".to_string(), "Curry".to_string()]
                ),
                (date!(2026 - 11 - 9), vec!["Pasta".to_string()]),
            ]
        );
    }

    #[test]
    fn each_recipe_is_counted_across_the_whole_log() {
        let pasta = Uuid::now_v7();
        let curry = Uuid::now_v7();
        let entries = vec![
            cook(pasta, "Pasta", date!(2026 - 11 - 14)),
            cook(curry, "Curry", date!(2026 - 11 - 12)),
            cook(pasta, "Pasta", date!(2026 - 11 - 9)),
        ];

        let counts = cook_counts(&entries);
        assert_eq!(counts[&pasta], 2);
        assert_eq!(counts[&curry], 1);
    }
}
