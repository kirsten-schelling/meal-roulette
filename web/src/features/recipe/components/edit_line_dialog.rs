use crate::components::ui::{Alert, Button, ButtonVariant, Checkbox, Input, Modal, Tone};
use crate::features::recipe::components::ingredient_composer::parse_qty;
use crate::features::recipe::hooks::use_recipe::update_recipe_ingredient;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::recipe_ingredient::{RecipeIngredient, UpdateRecipeIngredient};

/// Changes one saved ingredient line: its quantity, note, and whether it is optional.
#[component]
pub fn EditLineDialog(
    line: RecipeIngredient,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let mut qty = use_signal(|| line.qty.normalize().to_string());
    let mut note = use_signal(|| line.note.clone().unwrap_or_default());
    let mut optional = use_signal(|| line.is_optional);
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);

    let qty_check = parse_qty(&qty());
    let (recipe_id, ingredient_id) = (line.recipe_id, line.ingredient_id);
    let save = move |_| {
        let Ok(parsed) = parse_qty(&qty()) else {
            return;
        };
        if saving() {
            return;
        }
        let trimmed = note().trim().to_string();
        let update = UpdateRecipeIngredient {
            qty: parsed,
            note: (!trimmed.is_empty()).then_some(trimmed),
            is_optional: optional(),
        };
        saving.set(true);
        error.set(None);
        spawn(async move {
            match update_recipe_ingredient(recipe_id, ingredient_id, update).await {
                Ok(_) => on_saved.call(()),
                Err(e) => error.set(Some(error_message(&e))),
            }
            saving.set(false);
        });
    };

    rsx! {
        Modal {
            title: format!("Edit {}", line.ingredient_name),
            on_close: move |_| on_close.call(()),

            div { class: "mr-dialog-form",
                div { class: "mr-inline-add",
                    div { class: "mr-qty",
                        Input {
                            label: format!("Qty ({})", line.unit),
                            kind: "number".to_string(),
                            value: qty(),
                            required: true,
                            oninput: move |e: FormEvent| qty.set(e.value()),
                            error: qty_check.clone().err(),
                        }
                    }
                    Input {
                        label: "Note (optional)".to_string(),
                        value: note(),
                        oninput: move |e: FormEvent| note.set(e.value()),
                        placeholder: "e.g. finely chopped".to_string(),
                    }
                }
                Checkbox {
                    checked: optional(),
                    onchange: move |v: bool| optional.set(v),
                    label: "Optional".to_string(),
                }
                if let Some(msg) = error() {
                    Alert { tone: Tone::Red, title: "Couldn't save the line", "{msg}" }
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
                    loading: saving(),
                    disabled: qty_check.is_err(),
                    onclick: save,
                    "Save"
                }
            }
        }
    }
}
