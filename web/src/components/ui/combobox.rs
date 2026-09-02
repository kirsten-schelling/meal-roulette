use crate::components::ui::Field;
use dioxus::prelude::*;
use uuid::Uuid;

/// One row offered by a `Combobox`. `hint` is optional right-aligned muted
/// text (e.g. an ingredient's unit).
#[derive(Clone, PartialEq, Default)]
pub struct ComboOption {
    pub id: Uuid,
    pub label: String,
    pub hint: String,
}

impl ComboOption {
    pub fn new(id: impl Into<Uuid>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            hint: String::new(),
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
/// `on_manage` pins a quiet "manage this list" row to the bottom of the popup
/// — a portal to wherever the catalog is maintained.
#[component]
pub fn Combobox(
    label: String,
    placeholder: String,
    options: Vec<ComboOption>,
    mut picker: Signal<Picker>,
    on_create: Option<EventHandler<String>>,
    #[props(default = true)] can_create: bool,
    manage_label: Option<String>,
    on_manage: Option<EventHandler<()>>,
) -> Element {
    let mut open = use_signal(|| false);

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

    let combo_cls = match (open(), is_new) {
        (true, true) => "mr-combo mr-combo--open mr-combo--flagged",
        (true, false) => "mr-combo mr-combo--open",
        (false, true) => "mr-combo mr-combo--flagged",
        (false, false) => "mr-combo",
    };

    let kd_trimmed = trimmed.clone();
    let kd_exact = exact.clone();
    let create_trimmed = trimmed.clone();

    rsx! {
        Field { label: label.clone(),
            div { class: "{combo_cls}",
                input {
                    class: "mr-input",
                    r#type: "text",
                    value: "{text}",
                    placeholder: "{placeholder}",
                    aria_label: "{label}",
                    onclick: move |_| open.set(true),
                    oninput: move |e| {
                        picker.write().type_text(e.value());
                        open.set(true);
                    },
                    onkeydown: move |e: KeyboardEvent| match e.key() {
                        Key::Enter => {
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
                        Key::Escape => open.set(false),
                        _ => {}
                    },
                }
                if is_new {
                    span { class: "mr-combo__flag", "New" }
                }
                if open() {
                    div { class: "mr-sel__backdrop", onclick: move |_| open.set(false) }
                    div { class: "mr-sel__popup",
                        div { class: "mr-sel__list mr-scroll", role: "listbox",
                            for o in filtered.iter() {
                                {
                                    let opt = o.clone();
                                    rsx! {
                                        div {
                                            key: "{o.id}",
                                            class: "mr-sel__opt",
                                            role: "option",
                                            onclick: move |_| {
                                                picker.write().pick(opt.clone());
                                                open.set(false);
                                            },
                                            span { "{o.label}" }
                                            if !o.hint.is_empty() {
                                                span { class: "mr-combo__unit", "{o.hint}" }
                                            }
                                        }
                                    }
                                }
                            }
                            if show_create {
                                div {
                                    class: "mr-sel__opt mr-combo__create",
                                    role: "option",
                                    onclick: move |_| {
                                        picker.write().create(create_trimmed.clone());
                                        if let Some(cb) = &on_create {
                                            cb.call(create_trimmed.clone());
                                        }
                                        open.set(false);
                                    },
                                    "＋ Create “{trimmed}”"
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
