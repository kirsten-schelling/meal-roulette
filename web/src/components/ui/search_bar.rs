use crate::components::ui::Chip;
use dioxus::prelude::*;
use std::rc::Rc;

/// One filter type offered in the search bar's `+` menu — e.g. "Tag", "Cuisine".
/// `placeholder` is shown while you type that filter's value.
#[derive(Clone, PartialEq)]
pub struct FilterField {
    pub label: String,
    pub placeholder: String,
}

impl FilterField {
    pub fn new(label: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            placeholder: placeholder.into(),
        }
    }
}

/// A search bar that doubles as a filter builder.
///
/// Type to search. If you pass `filters`, a `+` appears that opens a menu of
/// filter types; pick one, type a value, press Enter, and it becomes a chip in
/// the bar. Backspace on an empty field peels the last chip back off. With no
/// `filters`, it's just a plain search box.
///
/// It manages its own display state (chips, text, the open menu) so it's usable
/// on its own; wire it into real behaviour with the optional `onquery` (free
/// text) and `onfilters` (the applied chip labels) handlers.
#[component]
pub fn SearchBar(
    /// Filter types for the `+` menu. Empty (default) = a plain search box.
    #[props(default)]
    filters: Vec<FilterField>,
    #[props(default = "Search…".to_string())] placeholder: String,
    /// Fires with the free-text query on every keystroke.
    #[props(optional)]
    onquery: Option<EventHandler<String>>,
    /// Fires with the applied filter chip labels whenever they change.
    #[props(optional)]
    onfilters: Option<EventHandler<Vec<String>>>,
) -> Element {
    let mut chips = use_signal(Vec::<String>::new);
    let mut text = use_signal(String::new);
    // The filter you picked from `+` and are now typing a value for.
    let mut pending = use_signal(|| None as Option<FilterField>);
    let mut menu = use_signal(|| false);
    // Handle to the text field so picking a filter can focus it immediately.
    let mut input_el = use_signal(|| None as Option<Rc<MountedData>>);

    let has_filters = !filters.is_empty();

    let ph = if let Some(f) = pending() {
        f.placeholder.clone()
    } else if chips().is_empty() {
        placeholder.clone()
    } else if has_filters {
        "Search, or + to add a filter…".to_string()
    } else {
        placeholder.clone()
    };

    rsx! {
        div { class: "mr-fsearch",
            span { class: "mr-fsearch__icon", aria_hidden: "true" }

            // Applied filters, as removable chips inside the bar.
            for c in chips().iter() {
                {
                    let label = c.clone();
                    rsx! {
                        Chip {
                            key: "{c}",
                            label: "{c}",
                            onremove: move |_| {
                                let mut cur = chips();
                                cur.retain(|x| x != &label);
                                chips.set(cur.clone());
                                if let Some(cb) = onfilters {
                                    cb.call(cur);
                                }
                            },
                        }
                    }
                }
            }

            // The filter you're currently filling in (e.g. "Difficulty:").
            if let Some(f) = pending() {
                span { class: "mr-fsearch__pending", "{f.label}:" }
            }

            input {
                class: "mr-fsearch__input",
                r#type: "text",
                value: "{text}",
                placeholder: "{ph}",
                aria_label: "Search",
                onmounted: move |e| input_el.set(Some(e.data())),
                oninput: move |e| {
                    let v = e.value();
                    text.set(v.clone());
                    if let Some(cb) = onquery {
                        cb.call(v);
                    }
                },
                onkeydown: move |e: KeyboardEvent| match e.key() {
                    // Enter commits the typed value as a chip (only with a pending filter).
                    Key::Enter => {
                        if let Some(f) = pending() {
                            let value = text().trim().to_string();
                            if !value.is_empty() {
                                let label = format!("{}: {}", f.label, value);
                                let mut cur = chips();
                                if !cur.contains(&label) {
                                    cur.push(label);
                                    chips.set(cur.clone());
                                    if let Some(cb) = onfilters {
                                        cb.call(cur);
                                    }
                                }
                                text.set(String::new());
                                pending.set(None);
                            }
                        }
                    }
                    // Backspace on an empty field peels back the pending filter, else the last chip.
                    Key::Backspace => {
                        if text().is_empty() {
                            if pending().is_some() {
                                pending.set(None);
                            } else if !chips().is_empty() {
                                let mut cur = chips();
                                cur.pop();
                                chips.set(cur.clone());
                                if let Some(cb) = onfilters {
                                    cb.call(cur);
                                }
                            }
                        }
                    }
                    Key::Escape => {
                        pending.set(None);
                        menu.set(false);
                    }
                    _ => {}
                },
            }

            // The "+" that opens the list of filter types (only when there are filters).
            if has_filters {
                button {
                    r#type: "button",
                    class: "mr-fsearch__add",
                    aria_haspopup: "listbox",
                    aria_expanded: menu(),
                    aria_label: "Add a filter",
                    onclick: move |_| {
                        let now = menu();
                        menu.set(!now);
                    },
                    "+"
                }

                if menu() {
                    div {
                        class: "mr-sel__backdrop",
                        onclick: move |_| menu.set(false),
                    }
                    div { class: "mr-fsearch__menu",
                        div { class: "mr-fsearch__menu-head", "Filter by" }
                        div { class: "mr-sel__list mr-scroll", role: "listbox",
                            for f in filters.iter() {
                                {
                                    let field = f.clone();
                                    rsx! {
                                        div {
                                            key: "{f.label}",
                                            class: "mr-sel__opt",
                                            role: "option",
                                            onclick: move |_| {
                                                pending.set(Some(field.clone()));
                                                menu.set(false);
                                                text.set(String::new());
                                                // focus the field so you can type the value right away
                                                if let Some(el) = input_el() {
                                                    spawn(async move {
                                                        let _ = el.set_focus(true).await;
                                                    });
                                                }
                                            },
                                            span { "{f.label}" }
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
