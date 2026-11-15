use crate::components::ui::{Alert, Button, Card, ComboOption, Combobox, Input, Picker, Tone};
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::pantry::components::pantry_row::parse_qty;
use crate::features::pantry::hooks::use_pantry::set_pantry_qty;
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use dioxus::prelude::*;

/// Sets the count of an ingredient the pantry does not list yet, or replaces the one it does.
#[component]
pub fn AddToPantry(on_saved: EventHandler<()>) -> Element {
    let ingredients = use_get_ingredients();
    let toast = use_toast();
    let mut ingredient = use_signal(Picker::default);
    let mut qty_text = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let ingredient_id = ingredient.read().picked.as_ref().map(|option| option.id);
    let qty = parse_qty(&qty_text());
    let options: Vec<ComboOption> = match ingredients() {
        Some(Ok(list)) => list
            .iter()
            .map(|i| ComboOption::new(i.id, i.name.clone()).with_hint(i.unit.clone()))
            .collect(),
        _ => Vec::new(),
    };

    let submit = move |_| {
        let (Some(ingredient_id), Some(qty)) = (ingredient_id, qty) else {
            return;
        };
        saving.set(true);
        spawn(async move {
            match set_pantry_qty(ingredient_id, qty).await {
                Ok(_) => {
                    ingredient.set(Picker::default());
                    qty_text.set(String::new());
                    on_saved.call(());
                }
                Err(e) => toast.show(
                    Some("Couldn't save the count".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            saving.set(false);
        });
    };

    rsx! {
        Card { header: "Set a count".to_string(),
            if let Some(Err(e)) = ingredients() {
                Alert { tone: Tone::Red, title: "Couldn't load ingredients", "{error_message(&e)}" }
            }
            div { class: "mr-logcook",
                Combobox {
                    label: "Ingredient".to_string(),
                    placeholder: "Search ingredients…".to_string(),
                    options,
                    picker: ingredient,
                    can_create: false,
                }
                Input {
                    label: "How much you have".to_string(),
                    inputmode: "decimal".to_string(),
                    enterkeyhint: "done".to_string(),
                    value: qty_text(),
                    oninput: move |e: FormEvent| qty_text.set(e.value()),
                }
                Button {
                    full_width: true,
                    loading: saving(),
                    disabled: ingredient_id.is_none() || qty.is_none(),
                    onclick: submit,
                    "Save count"
                }
            }
        }
    }
}
