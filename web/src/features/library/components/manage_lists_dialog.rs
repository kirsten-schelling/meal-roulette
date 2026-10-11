use crate::components::ui::{
    Alert, Button, ButtonVariant, ConfirmDialog, Icon, IconButton, IconName, Modal, Popover, Size,
    Spinner, Tabs, Text, Tone,
};
use crate::features::cuisine::hooks::use_cuisine::{
    use_add_cuisine, use_delete_cuisine, use_edit_cuisine, use_get_cuisines,
};
use crate::features::protein::hooks::use_protein::{
    use_add_protein, use_delete_protein, use_edit_protein, use_get_proteins,
};
use crate::features::tag::hooks::use_tag::{
    use_add_tag, use_delete_tag, use_edit_tag, use_get_tags,
};
use crate::features::unit::hooks::use_unit::{
    use_add_unit, use_delete_unit, use_edit_unit, use_get_units,
};
use crate::models::error_message;
use dioxus::prelude::*;
use domain::cuisine::{NewCuisine, UpdateCuisine};
use domain::protein::{NewProtein, UpdateProtein};
use domain::tag::{NewTag, UpdateTag};
use domain::unit::{NewUnit, UpdateUnit};
use domain::usage::UsageRef;
use uuid::Uuid;

/// Which list the dialog opens on. Callers deep-link straight to the list
/// they came from (a combobox portal, a library card).
#[derive(Clone, Copy, PartialEq)]
pub enum ManageTab {
    Cuisines,
    Proteins,
    Units,
    Tags,
}

impl ManageTab {
    fn index(self) -> usize {
        match self {
            ManageTab::Cuisines => 0,
            ManageTab::Proteins => 1,
            ManageTab::Units => 2,
            ManageTab::Tags => 3,
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            1 => ManageTab::Proteins,
            2 => ManageTab::Units,
            3 => ManageTab::Tags,
            _ => ManageTab::Cuisines,
        }
    }
}

/// One entry of whichever list is showing, flattened to what the rows render.
/// `hint` is optional muted detail (a unit's description). `used_by` names
/// what still references the entry; `None` until usage comes from the API.
#[derive(Clone, PartialEq)]
struct Row {
    id: Uuid,
    name: String,
    hint: String,
    used_by: Option<Vec<String>>,
}

fn usage_names(refs: &[UsageRef]) -> Vec<String> {
    refs.iter().map(|r| r.name.clone()).collect()
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
        Some(Err(e)) => Some(Err(error_message(e))),
    }
}

/// The error side of a mutation result, as a plain message for an `Alert`.
fn error_of<T>(result: &Option<Result<T, ServerFnError>>) -> Option<String> {
    match result {
        Some(Err(e)) => Some(error_message(e)),
        _ => None,
    }
}

/// Runs `then` each time a mutation's result lands as `Ok` — the shared
/// "on success, tidy up and refetch" wiring every list reuses.
fn use_on_success<T: 'static>(
    result: Signal<Option<Result<T, ServerFnError>>>,
    mut then: impl FnMut() + 'static,
) {
    use_effect(move || {
        if matches!(&*result.read(), Some(Ok(_))) {
            then();
        }
    });
}

/// The tabbed manager for the tiny lookup lists (cuisines / proteins / units / tags):
/// add, rename, delete. Mount it conditionally, like `Modal`.
#[component]
pub fn ManageListsDialog(initial_tab: ManageTab, on_close: EventHandler<()>) -> Element {
    let mut tab = use_signal(|| initial_tab.index());

    let mut cuisines = use_get_cuisines();
    let mut proteins = use_get_proteins();
    let mut units = use_get_units();
    let mut tags = use_get_tags();

    let add_cuisine = use_add_cuisine();
    let edit_cuisine = use_edit_cuisine();
    let delete_cuisine = use_delete_cuisine();
    let add_protein = use_add_protein();
    let edit_protein = use_edit_protein();
    let delete_protein = use_delete_protein();
    let add_unit = use_add_unit();
    let edit_unit = use_edit_unit();
    let delete_unit = use_delete_unit();
    let add_tag = use_add_tag();
    let edit_tag = use_edit_tag();
    let delete_tag = use_delete_tag();

    // The add row's draft; cleared on tab switch and on a successful add.
    let mut new_name = use_signal(String::new);

    // Row id + draft text while a name is being edited (one at a time).
    let mut editing = use_signal(|| None as Option<(Uuid, String)>);
    // Row id + name awaiting delete confirmation.
    let mut confirming = use_signal(|| None as Option<(Uuid, String)>);

    // A successful add clears the draft, a successful rename closes the
    // editor, and every success refetches the list it changed.
    use_on_success(add_cuisine.result, move || {
        new_name.set(String::new());
        cuisines.restart();
    });
    use_on_success(edit_cuisine.result, move || {
        editing.set(None);
        cuisines.restart();
    });
    use_on_success(delete_cuisine.result, move || cuisines.restart());
    use_on_success(add_protein.result, move || {
        new_name.set(String::new());
        proteins.restart();
    });
    use_on_success(edit_protein.result, move || {
        editing.set(None);
        proteins.restart();
    });
    use_on_success(delete_protein.result, move || proteins.restart());
    use_on_success(add_unit.result, move || {
        new_name.set(String::new());
        units.restart();
    });
    use_on_success(edit_unit.result, move || {
        editing.set(None);
        units.restart();
    });
    use_on_success(delete_unit.result, move || units.restart());
    use_on_success(add_tag.result, move || {
        new_name.set(String::new());
        tags.restart();
    });
    use_on_success(edit_tag.result, move || {
        editing.set(None);
        tags.restart();
    });
    use_on_success(delete_tag.result, move || tags.restart());

    // A mutation result outlives the action that produced it; drop them all
    // when the user moves to another tab or starts a new action, so a stale
    // error (or a re-fired success effect) never leaks across. Each signal is
    // copied to a local first so this stays an `Fn` closure — shareable across
    // the handlers below the way `submit_add` is.
    let clear_results = move || {
        let mut results = (
            add_cuisine.result,
            edit_cuisine.result,
            delete_cuisine.result,
        );
        results.0.set(None);
        results.1.set(None);
        results.2.set(None);
        let mut results = (
            add_protein.result,
            edit_protein.result,
            delete_protein.result,
        );
        results.0.set(None);
        results.1.set(None);
        results.2.set(None);
        let mut results = (add_unit.result, edit_unit.result, delete_unit.result);
        results.0.set(None);
        results.1.set(None);
        results.2.set(None);
        let mut results = (add_tag.result, edit_tag.result, delete_tag.result);
        results.0.set(None);
        results.1.set(None);
        results.2.set(None);
    };

    let current = ManageTab::from_index(tab());

    let rows = match current {
        ManageTab::Cuisines => rows_from(&cuisines(), |c| Row {
            id: c.cuisine.id,
            name: c.cuisine.name.clone(),
            hint: String::new(),
            used_by: Some(usage_names(&c.used_by)),
        }),
        ManageTab::Proteins => rows_from(&proteins(), |p| Row {
            id: p.protein.id,
            name: p.protein.name.clone(),
            hint: String::new(),
            used_by: Some(usage_names(&p.used_by)),
        }),
        ManageTab::Units => rows_from(&units(), |u| Row {
            id: u.unit.id,
            name: u.unit.name.clone(),
            hint: u.unit.description.clone().unwrap_or_default(),
            used_by: Some(usage_names(&u.used_by)),
        }),
        ManageTab::Tags => rows_from(&tags(), |t| Row {
            id: t.tag.id,
            name: t.tag.name.clone(),
            hint: String::new(),
            used_by: Some(usage_names(&t.used_by)),
        }),
    };

    let adding = match current {
        ManageTab::Cuisines => (add_cuisine.loading)(),
        ManageTab::Proteins => (add_protein.loading)(),
        ManageTab::Units => (add_unit.loading)(),
        ManageTab::Tags => (add_tag.loading)(),
    };
    let (add_error, rename_error, delete_error) = match current {
        ManageTab::Cuisines => (
            error_of(&add_cuisine.result.read()),
            error_of(&edit_cuisine.result.read()),
            error_of(&delete_cuisine.result.read()),
        ),
        ManageTab::Proteins => (
            error_of(&add_protein.result.read()),
            error_of(&edit_protein.result.read()),
            error_of(&delete_protein.result.read()),
        ),
        ManageTab::Units => (
            error_of(&add_unit.result.read()),
            error_of(&edit_unit.result.read()),
            error_of(&delete_unit.result.read()),
        ),
        ManageTab::Tags => (
            error_of(&add_tag.result.read()),
            error_of(&edit_tag.result.read()),
            error_of(&delete_tag.result.read()),
        ),
    };
    let add_placeholder = match current {
        ManageTab::Cuisines => "Add a cuisine…",
        ManageTab::Proteins => "Add a protein…",
        ManageTab::Units => "Add a unit…",
        ManageTab::Tags => "Add a tag…",
    };
    let used_by_noun = match current {
        ManageTab::Cuisines | ManageTab::Proteins | ManageTab::Tags => "recipe",
        ManageTab::Units => "ingredient",
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
            ManageTab::Tags => add_tag.submit(NewTag {
                household_id: None,
                name,
            }),
        }
    };

    let submit_rename = move |id: Uuid, name: String| match ManageTab::from_index(tab()) {
        ManageTab::Cuisines => edit_cuisine.submit(id, UpdateCuisine { name: Some(name) }),
        ManageTab::Proteins => edit_protein.submit(id, UpdateProtein { name: Some(name) }),
        ManageTab::Units => edit_unit.submit(
            id,
            UpdateUnit {
                name: Some(name),
                description: None,
            },
        ),
        ManageTab::Tags => edit_tag.submit(id, UpdateTag { name: Some(name) }),
    };

    let submit_delete = move |id: Uuid| match ManageTab::from_index(tab()) {
        ManageTab::Cuisines => delete_cuisine.submit(id),
        ManageTab::Proteins => delete_protein.submit(id),
        ManageTab::Units => delete_unit.submit(id),
        ManageTab::Tags => delete_tag.submit(id),
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
                        "Tags".to_string(),
                    ],
                    selected: tab(),
                    on_select: move |i: usize| {
                        tab.set(i);
                        editing.set(None);
                        confirming.set(None);
                        new_name.set(String::new());
                        clear_results();
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
                        Alert { tone: Tone::Red, title: "Couldn't add that", "{msg}" }
                    }
                    if let Some(msg) = rename_error {
                        Alert { tone: Tone::Red, title: "Couldn't rename that", "{msg}" }
                    }
                    if let Some(msg) = delete_error {
                        Alert { tone: Tone::Red, title: "Couldn't delete that", "{msg}" }
                    }

                    match rows {
                        None => rsx! {
                            div { class: "mr-manage__state", Spinner {} }
                        },
                        Some(Err(msg)) => rsx! {
                            Alert { tone: Tone::Red, title: "Couldn't load the list", "{msg}" }
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
                                        used_by_noun: used_by_noun.to_string(),
                                        editing_draft: editing()
                                            .filter(|(id, _)| *id == row.id)
                                            .map(|(_, draft)| draft),
                                        on_edit: move |(id, name): (Uuid, String)| {
                                            clear_results();
                                            editing.set(Some((id, name)));
                                        },
                                        on_draft: move |(id, draft): (Uuid, String)| {
                                            editing.set(Some((id, draft)));
                                        },
                                        on_cancel: move |_| editing.set(None),
                                        on_save: move |(id, name): (Uuid, String)| submit_rename(id, name),
                                        on_delete: move |(id, name): (Uuid, String)| {
                                            clear_results();
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

        if let Some((id, name)) = confirming() {
            ConfirmDeleteRow {
                name,
                used_by_noun: used_by_noun.to_string(),
                on_cancel: move |_| confirming.set(None),
                on_confirm: move |_| {
                    confirming.set(None);
                    submit_delete(id);
                },
            }
        }
    }
}

/// One list entry: name + optional hint, quiet rename/delete actions, and an
/// inline edit mode when `editing_draft` is set for it.
#[component]
fn ManageRow(
    row: Row,
    used_by_noun: String,
    editing_draft: Option<String>,
    on_edit: EventHandler<(Uuid, String)>,
    on_draft: EventHandler<(Uuid, String)>,
    on_cancel: EventHandler<()>,
    on_save: EventHandler<(Uuid, String)>,
    on_delete: EventHandler<(Uuid, String)>,
) -> Element {
    let id = row.id;

    if let Some(draft) = editing_draft {
        let key_draft = draft.clone();
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
                    oninput: move |e| on_draft.call((id, e.value())),
                    onkeydown: move |e: KeyboardEvent| match e.key() {
                        Key::Enter => {
                            if !key_draft.trim().is_empty() {
                                on_save.call((id, key_draft.trim().to_string()));
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
                            on_save.call((id, save_draft.trim().to_string()));
                        }
                    },
                    Icon { name: IconName::Check }
                }
                IconButton {
                    label: "Cancel rename".to_string(),
                    onclick: move |_| on_cancel.call(()),
                    Icon { name: IconName::X }
                }
            }
        };
    }

    let edit_name = row.name.clone();
    let del_name = row.name.clone();

    rsx! {
        div { class: "mr-managelist__row",
            span { class: "mr-managelist__name", "{row.name}" }
            if !row.hint.is_empty() {
                span { class: "mr-managelist__hint", "{row.hint}" }
            }
            span { class: "mr-managelist__count",
                UsagePopover { used_by: row.used_by.clone(), noun: used_by_noun }
            }
            div { class: "mr-rowactions",
                IconButton {
                    label: "Rename {row.name}".to_string(),
                    onclick: move |_| on_edit.call((id, edit_name.clone())),
                    Icon { name: IconName::Edit }
                }
                IconButton {
                    label: "Delete {row.name}".to_string(),
                    danger: true,
                    onclick: move |_| on_delete.call((id, del_name.clone())),
                    Icon { name: IconName::X }
                }
            }
        }
    }
}

/// The row's usage count, hoverable: the card lists what still references
/// the entry. `noun` is the singular of what that is ("recipe", "ingredient").
#[component]
fn UsagePopover(used_by: Option<Vec<String>>, noun: String) -> Element {
    let count = used_by.as_ref().map(Vec::len);
    let label = match count {
        None => "Usage isn't available yet".to_string(),
        Some(0) => format!("Not used by any {noun}"),
        Some(1) => format!("Used by 1 {noun}"),
        Some(n) => format!("Used by {n} {noun}s"),
    };
    let shown = count.map_or("—".to_string(), |n| n.to_string());

    rsx! {
        Popover { label: label.clone(), trigger: rsx! { "{shown}" },
            match used_by {
                Some(names) if !names.is_empty() => rsx! {
                    p { class: "mr-popover__title", "{label}" }
                    ul { class: "mr-popover__list mr-scroll",
                        for (i, name) in names.iter().enumerate() {
                            li { key: "{i}", class: "mr-popover__item", "{name}" }
                        }
                    }
                },
                _ => rsx! {
                    p { class: "mr-popover__note", "{label}." }
                },
            }
        }
    }
}

/// The delete confirmation, split out so the main dialog body stays readable.
#[component]
fn ConfirmDeleteRow(
    name: String,
    used_by_noun: String,
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
                "This removes it for good. If any {used_by_noun} still uses it, the delete is refused."
            }
        }
    }
}
