use crate::components::ui::{Chip, ComboOption, Combobox, Picker};
use dioxus::prelude::*;
use domain::tag::{TagEntry, TagRef};
use uuid::Uuid;

/// One tag chosen for a recipe: an existing one (`id`) or a name to create on save.
#[derive(Clone, PartialEq, Debug)]
pub struct TagChoice {
    pub id: Option<Uuid>,
    pub name: String,
}

impl TagChoice {
    fn same(&self, other: &TagChoice) -> bool {
        match (self.id, other.id) {
            (Some(a), Some(b)) => a == b,
            _ => self.name.eq_ignore_ascii_case(&other.name),
        }
    }

    pub fn entry(&self) -> TagEntry {
        match self.id {
            Some(id) => TagEntry::Existing(id),
            None => TagEntry::New(self.name.clone()),
        }
    }
}

impl From<&TagRef> for TagChoice {
    fn from(tag: &TagRef) -> Self {
        TagChoice {
            id: Some(tag.id),
            name: tag.name.clone(),
        }
    }
}

/// Several tags at once: the combobox adds one per pick (or per new name + Enter) and
/// clears itself, and the chosen tags sit below it as removable chips.
#[component]
pub fn TagPicker(
    label: Option<String>,
    options: Vec<ComboOption>,
    mut selected: Signal<Vec<TagChoice>>,
    on_manage: EventHandler<()>,
) -> Element {
    let mut picker = use_signal(Picker::default);

    use_effect(move || {
        let current = picker();
        let choice = match (current.picked, current.new_name) {
            (Some(opt), _) => TagChoice {
                id: Some(opt.id),
                name: opt.label,
            },
            (None, Some(name)) => TagChoice { id: None, name },
            (None, None) => return,
        };
        if !selected.peek().iter().any(|t| t.same(&choice)) {
            selected.write().push(choice);
        }
        picker.set(Picker::default());
    });

    let chosen = selected();
    let available: Vec<ComboOption> = options
        .into_iter()
        .filter(|o| !chosen.iter().any(|t| t.id == Some(o.id)))
        .collect();

    rsx! {
        div { class: "mr-tagpicker",
            Combobox {
                label: label.clone(),
                placeholder: "Add a tag, or type a new one and press Enter…".to_string(),
                options: available,
                picker,
                manage_label: "Manage tags…".to_string(),
                on_manage: move |_| on_manage.call(()),
            }
            if !chosen.is_empty() {
                div { class: "mr-tagpicker__chips",
                    for tag in chosen.iter() {
                        {
                            let removed = tag.clone();
                            let label = if tag.id.is_none() {
                                format!("{} (new)", tag.name)
                            } else {
                                tag.name.clone()
                            };
                            rsx! {
                                Chip {
                                    key: "{tag.name}",
                                    label,
                                    onremove: move |_| selected.write().retain(|t| !t.same(&removed)),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
