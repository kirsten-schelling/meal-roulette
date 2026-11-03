use crate::components::ui::menu_nav::{is_nav_key, next_index, option_class, option_id, reveal};
use crate::components::ui::{field::use_control_id, Chip, Icon, IconName};
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
    /// Fires with the free-text query on every keystroke. While a filter's value is being
    /// typed, that text is not the query, so it isn't reported.
    #[props(optional)]
    onquery: Option<EventHandler<String>>,
    /// Fires with the applied filter chip labels whenever they change.
    #[props(optional)]
    onfilters: Option<EventHandler<Vec<String>>>,
    /// Bump this number to empty the bar (text, chips and any half-typed filter).
    #[props(default)]
    reset: u32,
) -> Element {
    let mut chips = use_signal(Vec::<String>::new);
    let mut text = use_signal(String::new);
    // The filter you picked from `+` and are now typing a value for.
    let mut pending = use_signal(|| None as Option<FilterField>);
    let mut menu = use_signal(|| false);
    // Handle to the text field so picking a filter can focus it immediately.
    let mut input_el = use_signal(|| None as Option<Rc<MountedData>>);

    use_effect(use_reactive(&reset, move |reset| {
        if reset > 0 {
            chips.set(Vec::new());
            text.set(String::new());
            pending.set(None);
        }
    }));

    let has_filters = !filters.is_empty();

    let menu_base = use_control_id();
    let menu_list_id = format!("{menu_base}-list");
    // The highlighted row of the filter menu, moved by the arrow keys.
    let mut menu_hl = use_signal(|| None as Option<usize>);
    let filter_count = filters.len();
    let menu_hl_now = menu_hl().filter(|i| *i < filter_count);
    let menu_active_id = menu_hl_now.map(|i| option_id(&menu_base, i));

    let pick_filters = filters.clone();
    let pick_filter = use_callback(move |i: usize| {
        let Some(field) = pick_filters.get(i).cloned() else {
            return;
        };
        pending.set(Some(field));
        menu.set(false);
        text.set(String::new());
        if let Some(cb) = onquery {
            cb.call(String::new());
        }
        // focus the field so you can type the value right away
        if let Some(el) = input_el() {
            spawn(async move {
                let _ = el.set_focus(true).await;
            });
        }
    });

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
                    if pending().is_none() {
                        if let Some(cb) = onquery {
                            cb.call(v);
                        }
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
                    aria_controls: "{menu_list_id}",
                    aria_activedescendant: menu_active_id,
                    aria_label: "Add a filter",
                    onclick: move |_| {
                        let now = menu();
                        menu.set(!now);
                        if !now {
                            menu_hl.set(Some(0));
                        }
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        let key = e.key();
                        if !menu() {
                            if matches!(key, Key::ArrowDown | Key::ArrowUp) {
                                e.prevent_default();
                                menu.set(true);
                                menu_hl.set(Some(0));
                            }
                            return;
                        }
                        match key {
                            Key::Escape => {
                                // only this menu closes, not a dialog around it
                                e.stop_propagation();
                                menu.set(false);
                            }
                            Key::Tab => menu.set(false),
                            Key::Enter => {
                                e.prevent_default();
                                if let Some(i) = menu_hl_now {
                                    pick_filter.call(i);
                                }
                            }
                            _ if is_nav_key(&key) => {
                                e.prevent_default();
                                let next = next_index(menu_hl_now, filter_count, &key);
                                menu_hl.set(next);
                                if let Some(i) = next {
                                    reveal(&option_id(&menu_base, i));
                                }
                            }
                            _ => {}
                        }
                    },
                    Icon { name: IconName::Plus }
                }

                if menu() {
                    div {
                        class: "mr-sel__backdrop",
                        onclick: move |_| menu.set(false),
                    }
                    div { class: "mr-fsearch__menu",
                        div { class: "mr-fsearch__menu-head", "Filter by" }
                        div {
                            id: "{menu_list_id}",
                            class: "mr-sel__list mr-scroll",
                            role: "listbox",
                            for (i , f) in filters.iter().enumerate() {
                                div {
                                    key: "{f.label}",
                                    id: "{option_id(&menu_base, i)}",
                                    class: option_class(false, menu_hl_now == Some(i)),
                                    role: "option",
                                    aria_selected: menu_hl_now == Some(i),
                                    onmousemove: move |_| {
                                        if menu_hl() != Some(i) {
                                            menu_hl.set(Some(i));
                                        }
                                    },
                                    onclick: move |_| pick_filter.call(i),
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
