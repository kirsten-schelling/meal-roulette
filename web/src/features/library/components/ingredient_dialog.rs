use crate::components::ui::{
    Alert, Button, ButtonVariant, ComboOption, Combobox, Input, Modal, Picker, Spinner, Tone,
};
use crate::features::ingredient::hooks::use_ingredient::{update_ingredient, use_add_ingredient};
use crate::features::library::components::{ManageListsDialog, ManageTab};
use crate::features::unit::hooks::use_unit::use_get_units;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::ingredient::{Ingredient, NewIngredient, UpdateIngredient};
use domain::unit::UnitEntry;

/// The Library's ingredient dialog: adds a new ingredient, or edits `existing`. The unit
/// field picks an existing unit or, like the recipe flow, takes a new name that is created
/// together with the ingredient; its manage portal is for editing the units list without
/// leaving here.
#[component]
pub fn IngredientDialog(
    existing: Option<Ingredient>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let editing_id = existing.as_ref().map(|i| i.id);
    let start = existing.clone();
    let mut name = use_signal(|| start.as_ref().map(|i| i.name.clone()).unwrap_or_default());
    let mut purchase_size = use_signal(|| {
        start
            .as_ref()
            .map(|i| i.purchase_size.to_string())
            .unwrap_or_default()
    });
    let unit = use_signal(|| {
        let mut picker = Picker::default();
        if let Some(i) = &start {
            picker.pick(ComboOption::new(i.unit_id, i.unit.clone()));
        }
        picker
    });
    let mut units = use_get_units();

    // The manage-lists dialog stacks over this one, opened on Units.
    let mut managing = use_signal(|| false);

    let add = use_add_ingredient();
    let mut updating = use_signal(|| false);
    let mut update_error = use_signal(|| None as Option<String>);
    use_effect(move || {
        if matches!(&*add.result.read(), Some(Ok(_))) {
            on_saved.call(());
        }
    });
    let saving = (add.loading)() || updating();
    let error = match &*add.result.read() {
        Some(Err(e)) => Some(error_message(e)),
        _ => update_error(),
    };

    let unit_options: Vec<ComboOption> = match units() {
        Some(Ok(list)) => list
            .iter()
            .map(|u| ComboOption::new(u.unit.id, u.unit.name.clone()))
            .collect(),
        _ => Vec::new(),
    };

    // The parts only exist once every field is valid; the button keys off them,
    // so nothing here can panic on half-filled input.
    let parts: Option<(String, i32, UnitEntry)> = {
        let n = name().trim().to_string();
        let size = purchase_size()
            .trim()
            .parse::<i32>()
            .ok()
            .filter(|s| *s > 0);
        let u = unit();
        match (n.is_empty(), size, u.is_resolved()) {
            (false, Some(size), true) => {
                Some((n, size, u.resolve(UnitEntry::Existing, UnitEntry::New)))
            }
            _ => None,
        }
    };
    let can_save = parts.is_some();

    let save = move |_| {
        let Some((name, purchase_size, unit)) = parts.clone() else {
            return;
        };
        match editing_id {
            None => add.submit(NewIngredient {
                household_id: None,
                name,
                purchase_size,
                unit,
            }),
            Some(id) => {
                updating.set(true);
                update_error.set(None);
                spawn(async move {
                    let update = UpdateIngredient {
                        name,
                        purchase_size,
                        unit,
                    };
                    match update_ingredient(id, update).await {
                        Ok(_) => on_saved.call(()),
                        Err(e) => update_error.set(Some(error_message(&e))),
                    }
                    updating.set(false);
                });
            }
        }
    };

    let (title, save_label, error_title) = if editing_id.is_some() {
        (
            "Edit ingredient",
            "Save changes",
            "Couldn't save the ingredient",
        )
    } else {
        (
            "Add ingredient",
            "Save ingredient",
            "Couldn't add the ingredient",
        )
    };

    rsx! {
        Modal {
            title: title.to_string(),
            overflow_visible: true,
            on_close: move |_| on_close.call(()),

            div { class: "mr-dialog-form",
                Input {
                    label: "Name".to_string(),
                    value: name(),
                    required: true,
                    oninput: move |e: FormEvent| name.set(e.value()),
                    placeholder: "e.g. Coconut milk".to_string(),
                }
                div { class: "mr-form-grid",
                    Combobox {
                        label: "Unit".to_string(),
                        placeholder: "Search, or type + Enter…".to_string(),
                        options: unit_options.clone(),
                        picker: unit,
                        required: true,
                        manage_label: "Manage units…".to_string(),
                        on_manage: move |_| managing.set(true),
                    }
                    Input {
                        label: "Purchase size".to_string(),
                        kind: "number".to_string(),
                        value: purchase_size(),
                        required: true,
                        oninput: move |e: FormEvent| purchase_size.set(e.value()),
                        placeholder: "e.g. 400".to_string(),
                    }
                }

                if let Some(msg) = error {
                    Alert { tone: Tone::Red, title: error_title, "{msg}" }
                }
            }

            div { class: "mr-dialog-foot",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Solid,
                    disabled: saving || !can_save,
                    onclick: save,
                    if saving {
                        Spinner {}
                    } else {
                        "{save_label}"
                    }
                }
            }
        }

        if managing() {
            ManageListsDialog {
                initial_tab: ManageTab::Units,
                on_close: move |_| {
                    managing.set(false);
                    // A unit may have been added in there for this ingredient.
                    units.restart();
                },
            }
        }
    }
}
