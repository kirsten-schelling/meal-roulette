use crate::components::ui::{
    Alert, Button, ButtonVariant, ConfirmDialog, IconButton, Modal, Size, Spinner, Tabs, Text, Tone,
};
use crate::features::cuisine::hooks::use_cuisine::{use_add_cuisine, use_get_cuisines};
use crate::features::protein::hooks::use_protein::{use_add_protein, use_get_proteins};
use crate::features::unit::hooks::use_unit::{use_add_unit, use_get_units};
use dioxus::prelude::*;
use domain::cuisine::NewCuisine;
use domain::protein::NewProtein;
use domain::unit::NewUnit;

/// Which list the dialog opens on. Callers deep-link straight to the list
/// they came from (a combobox portal, a library card).
#[derive(Clone, Copy, PartialEq)]
pub enum ManageTab {
    Cuisines,
    Proteins,
    Units,
}

impl ManageTab {
    fn index(self) -> usize {
        match self {
            ManageTab::Cuisines => 0,
            ManageTab::Proteins => 1,
            ManageTab::Units => 2,
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            1 => ManageTab::Proteins,
            2 => ManageTab::Units,
            _ => ManageTab::Cuisines,
        }
    }
}

/// One entry of whichever list is showing, flattened to what the rows render.
/// `hint` is optional muted detail (a unit's description).
#[derive(Clone, PartialEq)]
struct Row {
    id: String,
    name: String,
    hint: String,
}

/// Flattens a lookup fetch into rows — the error becomes a plain message so
/// the match below stays simple.
fn rows_from<T>(
    fetched: &Option<Result<Vec<T>, ServerFnError>>,
    to_row: impl Fn(&T) -> Row,
) -> Option<Result<Vec<Row>, String>> {
    match fetched {
        None => None,
        Some(Ok(items)) => Some(Ok(items.iter().map(to_row).collect())),
        Some(Err(e)) => Some(Err(e.to_string())),
    }
}

/// The tabbed manager for the tiny lookup lists (cuisines / proteins / units):
/// add, rename, delete. Mount it conditionally, like `Modal`.
#[component]
pub fn ManageListsDialog(initial_tab: ManageTab, on_close: EventHandler<()>) -> Element {
    let mut tab = use_signal(|| initial_tab.index());

    let mut cuisines = use_get_cuisines();
    let mut proteins = use_get_proteins();
    let mut units = use_get_units();

    let mut add_cuisine = use_add_cuisine();
    let mut add_protein = use_add_protein();
    let mut add_unit = use_add_unit();

    // The add row's draft; cleared on tab switch and on a successful add.
    let mut new_name = use_signal(String::new);

    // Row id + draft text while a name is being edited (one at a time).
    let mut editing = use_signal(|| None as Option<(String, String)>);
    // Row id + name awaiting delete confirmation.
    let mut confirming = use_signal(|| None as Option<(String, String)>);

    // A successful add clears the draft and refreshes its list.
    use_effect(move || {
        if matches!(&*add_cuisine.result.read(), Some(Ok(_))) {
            new_name.set(String::new());
            cuisines.restart();
        }
    });
    use_effect(move || {
        if matches!(&*add_protein.result.read(), Some(Ok(_))) {
            new_name.set(String::new());
            proteins.restart();
        }
    });
    use_effect(move || {
        if matches!(&*add_unit.result.read(), Some(Ok(_))) {
            new_name.set(String::new());
            units.restart();
        }
    });

    let current = ManageTab::from_index(tab());

    let rows = match current {
        ManageTab::Cuisines => rows_from(&cuisines(), |c| Row {
            id: c.id.to_string(),
            name: c.name.clone(),
            hint: String::new(),
        }),
        ManageTab::Proteins => rows_from(&proteins(), |p| Row {
            id: p.id.to_string(),
            name: p.name.clone(),
            hint: String::new(),
        }),
        ManageTab::Units => rows_from(&units(), |u| Row {
            id: u.id.to_string(),
            name: u.name.clone(),
            hint: u.description.clone().unwrap_or_default(),
        }),
    };

    let adding = match current {
        ManageTab::Cuisines => (add_cuisine.loading)(),
        ManageTab::Proteins => (add_protein.loading)(),
        ManageTab::Units => (add_unit.loading)(),
    };
    let add_error = match current {
        ManageTab::Cuisines => match &*add_cuisine.result.read() {
            Some(Err(e)) => Some(e.to_string()),
            _ => None,
        },
        ManageTab::Proteins => match &*add_protein.result.read() {
            Some(Err(e)) => Some(e.to_string()),
            _ => None,
        },
        ManageTab::Units => match &*add_unit.result.read() {
            Some(Err(e)) => Some(e.to_string()),
            _ => None,
        },
    };
    let add_placeholder = match current {
        ManageTab::Cuisines => "Add a cuisine…",
        ManageTab::Proteins => "Add a protein…",
        ManageTab::Units => "Add a unit…",
    };

    let submit_add = move || {
        let name = new_name().trim().to_string();
        if name.is_empty() {
            return;
        }
        match ManageTab::from_index(tab()) {
            ManageTab::Cuisines => add_cuisine.submit(NewCuisine {
                household_id: None,
                name,
            }),
            ManageTab::Proteins => add_protein.submit(NewProtein {
                household_id: None,
                name,
            }),
            ManageTab::Units => add_unit.submit(NewUnit {
                household_id: None,
                name,
                description: None,
            }),
        }
    };

    rsx! {
        Modal {
            title: "Manage lists".to_string(),
            on_close: move |_| on_close.call(()),

            div { class: "mr-manage",
                Tabs {
                    labels: vec![
                        "Cuisines".to_string(),
                        "Proteins".to_string(),
                        "Units".to_string(),
                    ],
                    selected: tab(),
                    on_select: move |i: usize| {
                        tab.set(i);
                        editing.set(None);
                        confirming.set(None);
                        new_name.set(String::new());
                        // A stale result from another tab would leak its error
                        // (or re-trigger the success effects) over here.
                        add_cuisine.result.set(None);
                        add_protein.result.set(None);
                        add_unit.result.set(None);
                    },
                }

                // Fixed-height body so switching tabs never resizes the dialog;
                // the row list scrolls inside it instead.
                div { class: "mr-manage__content",
                    div { class: "mr-manage__add",
                        input {
                            class: "mr-input",
                            r#type: "text",
                            value: "{new_name}",
                            placeholder: "{add_placeholder}",
                            aria_label: "{add_placeholder}",
                            oninput: move |e| new_name.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    submit_add();
                                }
                            },
                        }
                        Button {
                            variant: ButtonVariant::Solid,
                            size: Size::Sm,
                            loading: adding,
                            disabled: adding || new_name().trim().is_empty(),
                            onclick: move |_| submit_add(),
                            "Add"
                        }
                    }

                    if let Some(msg) = add_error {
                        Alert { tone: Tone::Red, title: "Couldn't add that",
                            "{msg}"
                        }
                    }

                    match rows {
                        None => rsx! {
                            div { class: "mr-manage__state",
                                Spinner {}
                            }
                        },
                        Some(Err(msg)) => rsx! {
                            Alert { tone: Tone::Red, title: "Couldn't load the list",
                                "{msg}"
                            }
                        },
                        Some(Ok(list)) if list.is_empty() => rsx! {
                            div { class: "mr-manage__state",
                                p { class: "mr-empty-note", "Nothing here yet. Add the first one above." }
                            }
                        },
                        Some(Ok(list)) => rsx! {
                            div { class: "mr-managelist mr-scroll",
                                for row in list.iter() {
                                    ManageRow {
                                        key: "{row.id}",
                                        row: row.clone(),
                                        editing_draft: editing()
                                            .filter(|(id, _)| *id == row.id)
                                            .map(|(_, draft)| draft),
                                        on_edit: move |(id, name): (String, String)| {
                                            editing.set(Some((id, name)));
                                        },
                                        on_draft: move |(id, draft): (String, String)| {
                                            editing.set(Some((id, draft)));
                                        },
                                        on_cancel: move |_| editing.set(None),
                                        // Display-only until the rename wiring lands —
                                        // saving just closes the editor for now.
                                        on_save: move |_: (String, String)| {
                                            editing.set(None);
                                        },
                                        on_delete: move |(id, name): (String, String)| {
                                            confirming.set(Some((id, name)));
                                        },
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }

        if let Some((_id, name)) = confirming() {
            ConfirmDeleteRow {
                name,
                on_cancel: move |_| confirming.set(None),
                // Display-only until the delete wiring lands — confirming just
                // closes the dialog for now.
                on_confirm: move |_| confirming.set(None),
            }
        }
    }
}

/// One list entry: name + optional hint, quiet rename/delete actions, and an
/// inline edit mode when `editing_draft` is set for it.
#[component]
fn ManageRow(
    row: Row,
    editing_draft: Option<String>,
    on_edit: EventHandler<(String, String)>,
    on_draft: EventHandler<(String, String)>,
    on_cancel: EventHandler<()>,
    on_save: EventHandler<(String, String)>,
    on_delete: EventHandler<(String, String)>,
) -> Element {
    if let Some(draft) = editing_draft {
        let id = row.id.clone();
        let key_id = row.id.clone();
        let key_draft = draft.clone();
        let save_id = row.id.clone();
        let save_draft = draft.clone();
        let can_save = !draft.trim().is_empty();

        return rsx! {
            div { class: "mr-managelist__row mr-managelist__row--editing",
                input {
                    class: "mr-input",
                    r#type: "text",
                    value: "{draft}",
                    aria_label: "New name",
                    autofocus: true,
                    oninput: move |e| on_draft.call((id.clone(), e.value())),
                    onkeydown: move |e: KeyboardEvent| match e.key() {
                        Key::Enter => {
                            if !key_draft.trim().is_empty() {
                                on_save.call((key_id.clone(), key_draft.trim().to_string()));
                            }
                        }
                        Key::Escape => on_cancel.call(()),
                        _ => {}
                    },
                }
                IconButton {
                    label: "Save name".to_string(),
                    onclick: move |_| {
                        if can_save {
                            on_save.call((save_id.clone(), save_draft.trim().to_string()));
                        }
                    },
                    "✓"
                }
                IconButton {
                    label: "Cancel rename".to_string(),
                    onclick: move |_| on_cancel.call(()),
                    "\u{00d7}"
                }
            }
        };
    }

    let edit_id = row.id.clone();
    let edit_name = row.name.clone();
    let del_id = row.id.clone();
    let del_name = row.name.clone();

    rsx! {
        div { class: "mr-managelist__row",
            span { class: "mr-managelist__name", "{row.name}" }
            if !row.hint.is_empty() {
                span { class: "mr-managelist__hint", "{row.hint}" }
            }
            // Usage count slot — dashes until counts come from the API.
            span { class: "mr-managelist__count", "—" }
            div { class: "mr-rowactions",
                IconButton {
                    label: "Rename {row.name}".to_string(),
                    onclick: move |_| on_edit.call((edit_id.clone(), edit_name.clone())),
                    "✎"
                }
                IconButton {
                    label: "Delete {row.name}".to_string(),
                    danger: true,
                    onclick: move |_| on_delete.call((del_id.clone(), del_name.clone())),
                    "\u{00d7}"
                }
            }
        }
    }
}

/// The delete confirmation, split out so the main dialog body stays readable.
#[component]
fn ConfirmDeleteRow(
    name: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        ConfirmDialog {
            title: "Delete “{name}”?".to_string(),
            confirm_label: "Delete".to_string(),
            danger: true,
            on_confirm: move |_| on_confirm.call(()),
            on_cancel: move |_| on_cancel.call(()),
            Text {
                "This removes it for good. If a recipe still uses it, the delete is refused."
            }
        }
    }
}
