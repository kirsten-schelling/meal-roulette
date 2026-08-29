use crate::components::ui::Field;
use dioxus::prelude::*;
use std::rc::Rc;

/// One entry in a `Select`. `value` is the stored value; `label` is shown.
#[derive(Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// A custom dropdown built from plain elements (not a native `<select>`).
///
/// - Plain (default): a button trigger showing the current selection.
/// - `searchable: true`: the trigger field itself becomes the search box —
///   clicking it clears the field, focuses it, and filters the list below as
///   you type. Clicking away without choosing reverts to the previous value.
///
/// Controlled: pass the selected `value`; `onchange` fires with the chosen value.
#[component]
pub fn Select(
    #[props(optional)] label: Option<String>,
    value: String,
    options: Vec<SelectOption>,
    onchange: EventHandler<String>,
    #[props(optional)] placeholder: Option<String>,
    #[props(default)] searchable: bool,
) -> Element {
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);
    // handle to the search field so we can focus it the instant the menu opens
    let mut field_el = use_signal(|| None as Option<Rc<MountedData>>);

    let current_label = options
        .iter()
        .find(|o| o.value == value)
        .map(|o| o.label.clone());
    let is_placeholder = current_label.is_none();
    let selected_text = current_label
        .clone()
        .or_else(|| placeholder.clone())
        .unwrap_or_else(|| "Select…".to_string());

    // Filter (only narrows when searchable + a query is typed).
    let q = query().to_lowercase();
    let filtered: Vec<SelectOption> = options
        .iter()
        .filter(|o| !searchable || q.is_empty() || o.label.to_lowercase().contains(&q))
        .cloned()
        .collect();

    rsx! {
        Field { label: label.clone(),
            if searchable {
                // ── field-as-search: the bar itself is the search input ──────
                {
                    let field_value = if open() { query() } else { selected_text.clone() };
                    let field_cls = if is_placeholder && !open() {
                        "mr-input mr-selb__field mr-selb__field--placeholder"
                    } else {
                        "mr-input mr-selb__field"
                    };
                    let wrap_cls = if open() { "mr-selb mr-selb--open" } else { "mr-selb" };
                    let ph = placeholder.clone().unwrap_or_else(|| "Search…".to_string());
                    let aria = label.clone().unwrap_or_else(|| "Select".to_string());
                    rsx! {
                        div { class: "{wrap_cls}",
                            input {
                                class: "{field_cls}",
                                r#type: "text",
                                readonly: !open(),
                                value: "{field_value}",
                                placeholder: "{ph}",
                                aria_label: "{aria}",
                                onmounted: move |e| field_el.set(Some(e.data())),
                                onclick: move |_| {
                                    if !open() {
                                        open.set(true);
                                        query.set(String::new());
                                        // focus the field so you can type immediately
                                        if let Some(el) = field_el() {
                                            spawn(async move {
                                                let _ = el.set_focus(true).await;
                                            });
                                        }
                                    }
                                },
                                oninput: move |e| query.set(e.value()),
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Escape {
                                        open.set(false);
                                    }
                                },
                            }
                            span { class: "mr-selb__chev" }
                            if open() {
                                div { class: "mr-selb__backdrop", onclick: move |_| open.set(false) }
                                div { class: "mr-selb__popup",
                                    div { class: "mr-sel__list mr-scroll", role: "listbox",
                                        if filtered.is_empty() {
                                            div { class: "mr-sel__empty", "No matches" }
                                        }
                                        for opt in filtered.iter() {
                                            {
                                                let v = opt.value.clone();
                                                let selected = opt.value == value;
                                                let cls = if selected { "mr-sel__opt mr-sel__opt--active" } else { "mr-sel__opt" };
                                                rsx! {
                                                    div {
                                                        key: "{opt.value}",
                                                        class: "{cls}",
                                                        role: "option",
                                                        aria_selected: selected,
                                                        onclick: move |_| {
                                                            onchange.call(v.clone());
                                                            open.set(false);
                                                            query.set(String::new());
                                                        },
                                                        span { "{opt.label}" }
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
            } else {
                // ── plain: button trigger showing the selection ──────────────
                {
                    let trigger_cls = if is_placeholder {
                        "mr-sel__trigger mr-sel__trigger--placeholder"
                    } else {
                        "mr-sel__trigger"
                    };
                    let chevron_cls = if open() {
                        "mr-sel__chevron mr-sel__chevron--open"
                    } else {
                        "mr-sel__chevron"
                    };
                    rsx! {
                        div { class: "mr-sel",
                            button {
                                r#type: "button",
                                class: "{trigger_cls}",
                                aria_haspopup: "listbox",
                                aria_expanded: open(),
                                onclick: move |_| {
                                    let now = open();
                                    open.set(!now);
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Escape {
                                        open.set(false);
                                    }
                                },
                                span { "{selected_text}" }
                                span { class: "{chevron_cls}" }
                            }
                            if open() {
                                div { class: "mr-sel__backdrop", onclick: move |_| open.set(false) }
                                div { class: "mr-sel__popup",
                                    div { class: "mr-sel__list mr-scroll", role: "listbox",
                                        for opt in filtered.iter() {
                                            {
                                                let v = opt.value.clone();
                                                let selected = opt.value == value;
                                                let cls = if selected { "mr-sel__opt mr-sel__opt--active" } else { "mr-sel__opt" };
                                                rsx! {
                                                    div {
                                                        key: "{opt.value}",
                                                        class: "{cls}",
                                                        role: "option",
                                                        aria_selected: selected,
                                                        onclick: move |_| {
                                                            onchange.call(v.clone());
                                                            open.set(false);
                                                        },
                                                        span { "{opt.label}" }
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
        }
    }
}
