use crate::components::ui::{Alert, Badge, Chip, Spinner, Tone};
use dioxus::prelude::*;

/// A rail card for one tiny lookup list (cuisines / proteins / units): a
/// titled surface showing the values as chips, with a count in the header.
/// The header button and every chip fire `on_manage` — the whole card is a
/// doorway into the manage dialog, opened on this list's tab.
#[component]
pub fn LookupCard(
    title: String,
    names: Option<Result<Vec<String>, String>>,
    on_manage: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "mr-card mr-lookupcard",
            div { class: "mr-lookupcard__head",
                span { class: "mr-lookupcard__title", "{title}" }
                if let Some(Ok(list)) = &names {
                    Badge { "{list.len()}" }
                }
                button {
                    r#type: "button",
                    class: "mr-lookupcard__manage",
                    onclick: move |_| on_manage.call(()),
                    "Manage"
                }
            }
            div { class: "mr-lookupcard__body",
                match names {
                    None => rsx! {
                        div { class: "mr-lookupcard__state", Spinner {} }
                    },
                    Some(Err(msg)) => rsx! {
                        Alert { tone: Tone::Red, "{msg}" }
                    },
                    Some(Ok(list)) if list.is_empty() => rsx! {
                        p { class: "mr-empty-note", "None yet. Add some via Manage." }
                    },
                    Some(Ok(list)) => rsx! {
                        div { class: "mr-lookupcard__chips",
                            for name in list.iter() {
                                Chip {
                                    key: "{name}",
                                    label: name.clone(),
                                    onclick: move |_| on_manage.call(()),
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
