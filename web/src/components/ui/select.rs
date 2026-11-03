use crate::components::ui::menu_nav::{is_nav_key, next_index, option_class, option_id, reveal};
use crate::components::ui::{field::use_control_id, Field};
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
///
/// Keyboard: Arrow Up/Down, Home and End move a highlight through the options,
/// Enter (or Space on the plain trigger) picks it, Escape and Tab close the list.
/// Focus stays on the control; the highlighted option is named by
/// `aria-activedescendant`.
#[component]
pub fn Select(
    #[props(optional)] label: Option<String>,
    value: String,
    options: Vec<SelectOption>,
    onchange: EventHandler<String>,
    #[props(optional)] placeholder: Option<String>,
    #[props(default)] searchable: bool,
) -> Element {
    let id = use_control_id();
    let list_id = format!("{id}-list");
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let mut hl = use_signal(|| None as Option<usize>);
    // Space picks on keydown; its keyup would then click the trigger and reopen it.
    let mut swallow_space = use_signal(|| false);
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
    let len = filtered.len();
    // The highlight when the list opens: the current choice, else the first row.
    let start_hl = filtered
        .iter()
        .position(|o| o.value == value)
        .or((len > 0).then_some(0));
    let hl_now = hl().filter(|i| *i < len);
    let active_id = hl_now.map(|i| option_id(&id, i));

    let values: Vec<String> = filtered.iter().map(|o| o.value.clone()).collect();
    let pick = use_callback(move |i: usize| {
        if let Some(v) = values.get(i) {
            onchange.call(v.clone());
        }
        open.set(false);
        query.set(String::new());
    });

    let options_view = rsx! {
        for (i , opt) in filtered.iter().enumerate() {
            div {
                key: "{opt.value}",
                id: "{option_id(&id, i)}",
                class: option_class(opt.value == value, hl_now == Some(i)),
                role: "option",
                aria_selected: opt.value == value,
                onmousemove: move |_| {
                    if hl() != Some(i) {
                        hl.set(Some(i));
                    }
                },
                onclick: move |_| pick.call(i),
                span { "{opt.label}" }
            }
        }
    };

    rsx! {
        Field { label: label.clone(), control_id: id.clone(),
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
                    let open_ids = id.clone();
                    rsx! {
                        div { class: "{wrap_cls}",
                            input {
                                id: "{id}",
                                class: "{field_cls}",
                                r#type: "text",
                                role: "combobox",
                                autocomplete: "off",
                                aria_autocomplete: "list",
                                aria_expanded: open(),
                                aria_controls: "{list_id}",
                                aria_activedescendant: active_id.clone(),
                                readonly: !open(),
                                value: "{field_value}",
                                placeholder: "{ph}",
                                onmounted: move |e| field_el.set(Some(e.data())),
                                onclick: move |_| {
                                    if !open() {
                                        open.set(true);
                                        query.set(String::new());
                                        hl.set(start_hl);
                                        // focus the field so you can type immediately
                                        if let Some(el) = field_el() {
                                            spawn(async move {
                                                let _ = el.set_focus(true).await;
                                            });
                                        }
                                    }
                                },
                                oninput: move |e| {
                                    query.set(e.value());
                                    hl.set(None);
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    let key = e.key();
                                    if !open() {
                                        if matches!(key, Key::ArrowDown | Key::ArrowUp | Key::Enter) {
                                            e.prevent_default();
                                            open.set(true);
                                            query.set(String::new());
                                            hl.set(start_hl);
                                        }
                                        return;
                                    }
                                    match key {
                                        Key::Escape => {
                                            // only this menu closes, not a dialog around it
                                            e.stop_propagation();
                                            open.set(false);
                                        }
                                        Key::Tab => open.set(false),
                                        Key::Enter => {
                                            e.prevent_default();
                                            if let Some(i) = hl_now {
                                                pick.call(i);
                                            }
                                        }
                                        _ if is_nav_key(&key) => {
                                            e.prevent_default();
                                            let next = next_index(hl_now, len, &key);
                                            hl.set(next);
                                            if let Some(i) = next {
                                                reveal(&option_id(&open_ids, i));
                                            }
                                        }
                                        _ => {}
                                    }
                                },
                            }
                            span { class: "mr-selb__chev", aria_hidden: "true" }
                            if open() {
                                div { class: "mr-selb__backdrop", onclick: move |_| open.set(false) }
                                div { class: "mr-selb__popup",
                                    div {
                                        id: "{list_id}",
                                        class: "mr-sel__list mr-scroll",
                                        role: "listbox",
                                        if filtered.is_empty() {
                                            div { class: "mr-sel__empty", "No matches" }
                                        }
                                        {options_view}
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
                    let open_ids = id.clone();
                    rsx! {
                        div { class: "mr-sel",
                            button {
                                id: "{id}",
                                r#type: "button",
                                class: "{trigger_cls}",
                                role: "combobox",
                                aria_haspopup: "listbox",
                                aria_expanded: open(),
                                aria_controls: "{list_id}",
                                aria_activedescendant: active_id.clone(),
                                onclick: move |_| {
                                    let now = open();
                                    open.set(!now);
                                    if !now {
                                        hl.set(start_hl);
                                    }
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    let key = e.key();
                                    let is_space = matches!(&key, Key::Character(c) if c == " ");
                                    if !open() {
                                        if matches!(key, Key::ArrowDown | Key::ArrowUp) {
                                            e.prevent_default();
                                            open.set(true);
                                            hl.set(start_hl);
                                        }
                                        return;
                                    }
                                    match key {
                                        Key::Escape => {
                                            // only this menu closes, not a dialog around it
                                            e.stop_propagation();
                                            open.set(false);
                                        }
                                        Key::Tab => open.set(false),
                                        Key::Enter => {
                                            e.prevent_default();
                                            if let Some(i) = hl_now {
                                                pick.call(i);
                                            }
                                        }
                                        _ if is_space => {
                                            e.prevent_default();
                                            swallow_space.set(true);
                                            if let Some(i) = hl_now {
                                                pick.call(i);
                                            }
                                        }
                                        _ if is_nav_key(&key) => {
                                            e.prevent_default();
                                            let next = next_index(hl_now, len, &key);
                                            hl.set(next);
                                            if let Some(i) = next {
                                                reveal(&option_id(&open_ids, i));
                                            }
                                        }
                                        _ => {}
                                    }
                                },
                                onkeyup: move |e: KeyboardEvent| {
                                    if swallow_space() && matches!(e.key(), Key::Character(c) if c == " ") {
                                        e.prevent_default();
                                        swallow_space.set(false);
                                    }
                                },
                                span { "{selected_text}" }
                                span { class: "{chevron_cls}", aria_hidden: "true" }
                            }
                            if open() {
                                div { class: "mr-sel__backdrop", onclick: move |_| open.set(false) }
                                div { class: "mr-sel__popup",
                                    div {
                                        id: "{list_id}",
                                        class: "mr-sel__list mr-scroll",
                                        role: "listbox",
                                        {options_view}
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
