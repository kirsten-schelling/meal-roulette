use crate::components::ui::{
    Alert, Button, ButtonVariant, ComboOption, Combobox, Input, Modal, Picker, Spinner, Tone,
};
use crate::features::ingredient::hooks::use_ingredient::use_add_ingredient;
use crate::features::library::components::{ManageListsDialog, ManageTab};
use crate::features::unit::hooks::use_unit::use_get_units;
use dioxus::prelude::*;
use domain::ingredient::NewIngredient;
use uuid::Uuid;

/// The standalone "Add ingredient" dialog for the Library page. Unlike the
/// recipe flow (which can create a unit transactionally), `POST /ingredients`
/// takes an existing `unit_id` — so the unit field is a strict picker, and its
/// manage portal is the way to create a missing unit without leaving here.
#[component]
pub fn AddIngredientDialog(on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut purchase_size = use_signal(String::new);
    let mut units = use_get_units();
    let unit = use_signal(Picker::default);

    // The manage-lists dialog stacks over this one, opened on Units.
    let mut managing = use_signal(|| false);

    let add = use_add_ingredient();
    use_effect(move || {
        if matches!(&*add.result.read(), Some(Ok(_))) {
            on_saved.call(());
        }
    });
    let saving = (add.loading)();
    let error = match &*add.result.read() {
        Some(Err(e)) => Some(e.to_string()),
        _ => None,
    };

    let unit_options: Vec<ComboOption> = match units() {
        Some(Ok(list)) => list
            .iter()
            .map(|u| ComboOption::new(u.id.to_string(), u.name.clone()))
            .collect(),
        _ => Vec::new(),
    };

    // The payload only exists once every field is valid; the button keys off it,
    // so nothing here can panic on half-filled input.
    let payload: Option<NewIngredient> = {
        let n = name().trim().to_string();
        let size = purchase_size()
            .trim()
            .parse::<i32>()
            .ok()
            .filter(|s| *s > 0);
        let unit_id = unit()
            .picked
            .as_ref()
            .and_then(|o| o.id.parse::<Uuid>().ok());
        match (n.is_empty(), size, unit_id) {
            (false, Some(purchase_size), Some(unit_id)) => Some(NewIngredient {
                household_id: None,
                name: n,
                purchase_size,
                unit_id,
            }),
            _ => None,
        }
    };
    let can_save = payload.is_some();

    rsx! {
        Modal {
            title: "Add ingredient".to_string(),
            overflow_visible: true,
            on_close: move |_| on_close.call(()),

            div { class: "mr-dialog-form",
                Input {
                    label: "Name".to_string(),
                    value: name(),
                    oninput: move |e: FormEvent| name.set(e.value()),
                    placeholder: "e.g. Coconut milk".to_string(),
                }
                div { class: "mr-form-grid",
                    Combobox {
                        label: "Unit".to_string(),
                        placeholder: "Search units…".to_string(),
                        options: unit_options.clone(),
                        picker: unit,
                        can_create: false,
                        manage_label: "Manage units…".to_string(),
                        on_manage: move |_| managing.set(true),
                    }
                    Input {
                        label: "Purchase size".to_string(),
                        kind: "number".to_string(),
                        value: purchase_size(),
                        oninput: move |e: FormEvent| purchase_size.set(e.value()),
                        placeholder: "e.g. 400".to_string(),
                    }
                }

                if let Some(msg) = error {
                    Alert { tone: Tone::Red, title: "Couldn't add the ingredient",
                        "{msg}"
                    }
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
                    onclick: move |_| {
                        if let Some(p) = payload.clone() {
                            add.submit(p);
                        }
                    },
                    if saving {
                        Spinner {}
                    } else {
                        "Save ingredient"
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
