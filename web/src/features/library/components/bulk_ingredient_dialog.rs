use crate::components::ui::{
    Alert, Button, ButtonVariant, Input, Modal, Select, SelectOption, Spinner, Tone,
};
use crate::features::ingredient::hooks::use_ingredient::update_ingredients;
use crate::features::ingredient::parse::parse_purchase_size;
use crate::features::shopping_list::hooks::use_shopping_categories::use_shopping_categories;
use crate::features::unit::hooks::use_unit::use_get_units;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::ingredient::{BulkIngredientUpdate, FieldChange};
use uuid::Uuid;

/// The select value meaning "leave each ingredient as it is".
const KEEP: &str = "keep";

/// A select value as a change: `KEEP` leaves it, empty clears it, an id sets it.
fn id_change(value: &str) -> FieldChange<Option<Uuid>> {
    match value {
        KEEP => FieldChange::Keep,
        _ => FieldChange::Set(value.parse().ok()),
    }
}

/// Sets the unit, purchase size and shopping category of every ingredient in `ids` at
/// once. Each field starts on "leave as is", so only what is changed here changes.
#[component]
pub fn BulkIngredientDialog(
    ids: Vec<Uuid>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let units = use_get_units();
    let categories = use_shopping_categories();
    let mut unit = use_signal(|| KEEP.to_string());
    let mut purchase_size = use_signal(String::new);
    let mut category = use_signal(|| KEEP.to_string());
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);

    let mut unit_options = vec![
        SelectOption::new(KEEP, "Leave as is"),
        SelectOption::new("", "No unit"),
    ];
    if let Some(Ok(list)) = &*units.read() {
        unit_options.extend(
            list.iter()
                .map(|u| SelectOption::new(u.unit.id.to_string(), u.unit.name.clone())),
        );
    }
    let mut category_options = vec![
        SelectOption::new(KEEP, "Leave as is"),
        SelectOption::new("", "No category"),
    ];
    if let Some(Ok(list)) = &*categories.read() {
        category_options.extend(
            list.iter()
                .map(|c| SelectOption::new(c.id.to_string(), c.name.clone())),
        );
    }

    let size_check = parse_purchase_size(&purchase_size());
    let changes = match &size_check {
        Ok(size) => Some(BulkIngredientUpdate {
            unit_id: id_change(&unit()),
            purchase_size: size.map_or(FieldChange::Keep, FieldChange::Set),
            shopping_category_id: id_change(&category()),
        }),
        Err(_) => None,
    };
    let unchanged = changes.as_ref().is_some_and(|c| {
        c.unit_id == FieldChange::Keep
            && c.purchase_size == FieldChange::Keep
            && c.shopping_category_id == FieldChange::Keep
    });
    let count = ids.len();
    let can_save = changes.is_some() && !unchanged;

    let save = move |_| {
        let Some(changes) = changes.clone() else {
            return;
        };
        if saving() {
            return;
        }
        saving.set(true);
        error.set(None);
        let ids = ids.clone();
        spawn(async move {
            match update_ingredients(ids, changes).await {
                Ok(_) => on_saved.call(()),
                Err(e) => error.set(Some(error_message(&e))),
            }
            saving.set(false);
        });
    };

    rsx! {
        Modal {
            title: format!("Edit {count} ingredients"),
            focus_dialog: true,
            on_close: move |_| on_close.call(()),

            div { class: "mr-dialog-form",
                Select {
                    label: "Unit".to_string(),
                    value: unit(),
                    options: unit_options,
                    onchange: move |v: String| unit.set(v),
                }
                Input {
                    label: "Purchase size".to_string(),
                    kind: "number".to_string(),
                    value: purchase_size(),
                    oninput: move |e: FormEvent| purchase_size.set(e.value()),
                    placeholder: "Leave as is".to_string(),
                    error: size_check.clone().err(),
                }
                Select {
                    label: "Shopping category".to_string(),
                    value: category(),
                    options: category_options,
                    onchange: move |v: String| category.set(v),
                }

                if let Some(msg) = error() {
                    Alert { tone: Tone::Red, title: "Couldn't save the changes", "{msg}" }
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
                    disabled: saving() || !can_save,
                    onclick: save,
                    if saving() {
                        Spinner {}
                    } else {
                        "Apply to {count}"
                    }
                }
            }
        }
    }
}
