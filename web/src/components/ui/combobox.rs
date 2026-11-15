use crate::components::ui::menu_nav::{is_nav_key, next_index, option_class, option_id, reveal};
use crate::components::ui::{field::use_control_id, Field, Icon, IconName};
use dioxus::prelude::*;
use uuid::Uuid;

/// One row offered by a `Combobox`. `hint` is optional right-aligned muted
/// text (e.g. an ingredient's unit). A `pending` option is one the form will
/// create on save (typed as new elsewhere in it): it is tagged "New", and
/// picking it records a create rather than an existing id.
#[derive(Clone, PartialEq, Default)]
pub struct ComboOption {
    pub id: Uuid,
    pub label: String,
    pub hint: String,
    pub pending: bool,
}

impl ComboOption {
    pub fn new(id: impl Into<Uuid>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            hint: String::new(),
            pending: false,
        }
    }

    /// A not-yet-saved name, offered so it can be reused before the form is saved.
    pub fn pending(label: impl Into<String>) -> Self {
        Self {
            pending: true,
            ..Self::new(Uuid::nil(), label)
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
}

/// A search-or-create field's state: the text shown, plus the resolved choice —
/// a picked catalog option, or a `new_name` to create. The methods keep those
/// in sync so a stale pick can't hide behind newly-typed text.
#[derive(Clone, PartialEq, Default)]
pub struct Picker {
    pub query: String,
    pub picked: Option<ComboOption>,
    pub new_name: Option<String>,
}

impl Picker {
    pub fn type_text(&mut self, v: String) {
        self.query = v;
        self.picked = None;
        self.new_name = None;
    }

    pub fn pick(&mut self, opt: ComboOption) {
        if opt.pending {
            self.create(opt.label);
            return;
        }
        self.query = opt.label.clone();
        self.new_name = None;
        self.picked = Some(opt);
    }

    pub fn create(&mut self, name: String) {
        self.query = name.clone();
        self.picked = None;
        self.new_name = Some(name);
    }

    /// The field holds a name that will be created on submit.
    pub fn is_new(&self) -> bool {
        self.new_name.is_some()
    }

    /// Something is chosen — an existing option or a pending create.
    pub fn is_resolved(&self) -> bool {
        self.picked.is_some() || self.new_name.is_some()
    }

    pub fn resolve<T>(
        self,
        existing: impl FnOnce(Uuid) -> T,
        created: impl FnOnce(String) -> T,
    ) -> T {
        match (self.picked, self.new_name) {
            (Some(opt), _) => existing(opt.id),
            (None, Some(name)) => created(name),
            (None, None) => panic!("Picker is not resolved"),
        }
    }
}

/// A type-to-search field that can also create. The caller hands over a
/// `Signal<Picker>` and the combobox keeps it in sync: typing filters
/// `options`, clicking a row (or Enter on an exact match) picks it, and Enter
/// on an unknown name (or the "Create …" row) records a pending create —
/// flagged with a "New" tag in the field so the choice is visible.
/// `on_create` optionally observes creates, for callers that reveal follow-up
/// fields when one happens. `can_create: false` turns the field into a strict
/// picker: unknown names offer no "Create" row and Enter on one does nothing.
/// `on_manage` pins a quiet "manage this list" row to the bottom of the popup:
/// a portal to wherever the catalog is maintained. Without a `label` the field is
/// announced by its placeholder, for spots where a label is already shown beside it.
///
/// Keyboard: Arrow Up/Down, Home and End move a highlight through the rows (the
/// "Create" row included); Enter picks the highlighted row, or falls back to the
/// exact-match / create behaviour when nothing is highlighted. Escape and Tab close.
#[component]
pub fn Combobox(
    label: Option<String>,
    placeholder: String,
    options: Vec<ComboOption>,
    mut picker: Signal<Picker>,
    on_create: Option<EventHandler<String>>,
    #[props(default = true)] can_create: bool,
    #[props(default)] required: bool,
    manage_label: Option<String>,
    on_manage: Option<EventHandler<()>>,
    /// Label of the phone keyboard's Enter key (`next`, `done`, …).
    enterkeyhint: Option<String>,
) -> Element {
    let id = use_control_id();
    let list_id = format!("{id}-list");
    let mut open = use_signal(|| false);
    let mut hl = use_signal(|| None as Option<usize>);

    let text = picker.read().query.clone();
    let trimmed = text.trim().to_string();
    let q = trimmed.to_lowercase();
    let filtered: Vec<ComboOption> = options
        .iter()
        .filter(|o| q.is_empty() || o.label.to_lowercase().contains(&q))
        .cloned()
        .collect();
    let exact = options
        .iter()
        .find(|o| o.label.eq_ignore_ascii_case(&trimmed))
        .cloned();
    let show_create = can_create && !trimmed.is_empty() && exact.is_none();
    let is_new = picker.read().is_new();

    // Rows the arrow keys walk: the matches, then the "Create" row when offered.
    let rows = filtered.len() + usize::from(show_create);
    let create_row = filtered.len();
    let hl_now = hl().filter(|i| *i < rows);
    let active_id = hl_now.map(|i| option_id(&id, i));

    let combo_cls = match (open(), is_new) {
        (true, true) => "mr-combo mr-combo--open mr-combo--flagged",
        (true, false) => "mr-combo mr-combo--open",
        (false, true) => "mr-combo mr-combo--flagged",
        (false, false) => "mr-combo",
    };

    let aria = label.clone().unwrap_or_else(|| placeholder.clone());
    let kd_trimmed = trimmed.clone();
    let kd_exact = exact.clone();
    let key_id = id.clone();

    let pick_options = filtered.clone();
    let create_trimmed = trimmed.clone();
    let pick = use_callback(move |i: usize| {
        if let Some(opt) = pick_options.get(i) {
            picker.write().pick(opt.clone());
        } else {
            picker.write().create(create_trimmed.clone());
            if let Some(cb) = &on_create {
                cb.call(create_trimmed.clone());
            }
        }
        open.set(false);
    });

    rsx! {
        Field { label: label.clone(), required, control_id: id.clone(),
            div { class: "{combo_cls}",
                input {
                    id: "{id}",
                    class: "mr-input",
                    r#type: "text",
                    role: "combobox",
                    autocomplete: "off",
                    enterkeyhint,
                    aria_autocomplete: "list",
                    aria_expanded: open(),
                    aria_controls: "{list_id}",
                    aria_activedescendant: active_id,
                    aria_label: "{aria}",
                    required,
                    value: "{text}",
                    placeholder: "{placeholder}",
                    onclick: move |_| open.set(true),
                    oninput: move |e| {
                        picker.write().type_text(e.value());
                        hl.set(None);
                        open.set(true);
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        let key = e.key();
                        if is_nav_key(&key) {
                            e.prevent_default();
                            if open() {
                                let next = next_index(hl_now, rows, &key);
                                hl.set(next);
                                if let Some(i) = next {
                                    reveal(&option_id(&key_id, i));
                                }
                            } else {
                                open.set(true);
                            }
                            return;
                        }
                        match key {
                            Key::Enter => {
                                if open() {
                                    if let Some(i) = hl_now {
                                        e.prevent_default();
                                        pick.call(i);
                                        return;
                                    }
                                }
                                if kd_trimmed.is_empty() {
                                    return;
                                }
                                match &kd_exact {
                                    Some(o) => picker.write().pick(o.clone()),
                                    None => {
                                        if !can_create {
                                            return;
                                        }
                                        picker.write().create(kd_trimmed.clone());
                                        if let Some(cb) = &on_create {
                                            cb.call(kd_trimmed.clone());
                                        }
                                    }
                                }
                                open.set(false);
                            }
                            Key::Escape => {
                                // only this menu closes, not a dialog around it
                                if open() {
                                    e.stop_propagation();
                                    open.set(false);
                                }
                            }
                            Key::Tab => open.set(false),
                            _ => {}
                        }
                    },
                }
                if is_new {
                    span { class: "mr-combo__flag", "New" }
                }
                if open() {
                    div {
                        class: "mr-sel__backdrop",
                        onclick: move |_| open.set(false),
                    }
                    div { class: "mr-sel__popup",
                        div {
                            id: "{list_id}",
                            class: "mr-sel__list mr-scroll",
                            role: "listbox",
                            for (i , o) in filtered.iter().enumerate() {
                                div {
                                    key: "{o.id}-{i}",
                                    id: "{option_id(&id, i)}",
                                    class: option_class(false, hl_now == Some(i)),
                                    role: "option",
                                    aria_selected: hl_now == Some(i),
                                    onmousemove: move |_| {
                                        if hl() != Some(i) {
                                            hl.set(Some(i));
                                        }
                                    },
                                    onclick: move |_| pick.call(i),
                                    span { "{o.label}" }
                                    if o.pending {
                                        span { class: "mr-combo__flag mr-combo__flag--inline", "New" }
                                    } else if !o.hint.is_empty() {
                                        span { class: "mr-combo__unit", "{o.hint}" }
                                    }
                                }
                            }
                            if show_create {
                                div {
                                    id: "{option_id(&id, create_row)}",
                                    class: "{option_class(false, hl_now == Some(create_row))} mr-combo__create",
                                    role: "option",
                                    aria_selected: hl_now == Some(create_row),
                                    onmousemove: move |_| {
                                        if hl() != Some(create_row) {
                                            hl.set(Some(create_row));
                                        }
                                    },
                                    onclick: move |_| pick.call(create_row),
                                    Icon { name: IconName::Plus }
                                    "Create “{trimmed}”"
                                }
                            }
                        }
                        if let Some(manage) = on_manage {
                            {
                                let label = manage_label
                                    .clone()
                                    .unwrap_or_else(|| "Manage…".to_string());
                                rsx! {
                                    button {
                                        r#type: "button",
                                        class: "mr-combo__manage",
                                        onclick: move |_| {
                                            open.set(false);
                                            manage.call(());
                                        },
                                        "{label}"
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
