use crate::components::ui::{Alert, Button, ButtonVariant, Input, Modal, Spinner, Tone};
use crate::features::library::components::CategorySelect;
use crate::features::shopping_list::hooks::use_shopping_items::{
    add_shopping_item, update_shopping_item,
};
use crate::models::error_message;
use dioxus::prelude::*;
use domain::shopping_item::ShoppingItem;

/// Adds an item that is not an ingredient (bin bags, candles), or edits `existing`.
#[component]
pub fn OtherItemDialog(
    existing: Option<ShoppingItem>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let editing_id = existing.as_ref().map(|i| i.id);
    let mut name = use_signal(|| {
        existing
            .as_ref()
            .map(|i| i.name.clone())
            .unwrap_or_default()
    });
    let mut category = use_signal(|| existing.as_ref().and_then(|i| i.shopping_category_id));
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);

    let trimmed = name().trim().to_string();
    let save = move |_| {
        if trimmed.is_empty() || saving() {
            return;
        }
        let name = trimmed.clone();
        saving.set(true);
        error.set(None);
        spawn(async move {
            let saved = match editing_id {
                None => add_shopping_item(name, category()).await,
                Some(id) => update_shopping_item(id, name, category()).await,
            };
            match saved {
                Ok(_) => on_saved.call(()),
                Err(e) => error.set(Some(error_message(&e))),
            }
            saving.set(false);
        });
    };

    let title = if editing_id.is_some() {
        "Edit item"
    } else {
        "Add other item"
    };

    rsx! {
        Modal {
            title: title.to_string(),
            focus_dialog: true,
            on_close: move |_| on_close.call(()),

            div { class: "mr-dialog-form",
                Input {
                    label: "Name".to_string(),
                    value: name(),
                    required: true,
                    oninput: move |e: FormEvent| name.set(e.value()),
                    placeholder: "e.g. Bin bags".to_string(),
                }
                CategorySelect { value: category(), onchange: move |id| category.set(id) }

                if let Some(msg) = error() {
                    Alert { tone: Tone::Red, title: "Couldn't save the item", "{msg}" }
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
                    disabled: saving() || name().trim().is_empty(),
                    onclick: save,
                    if saving() {
                        Spinner {}
                    } else {
                        "Save item"
                    }
                }
            }
        }
    }
}
