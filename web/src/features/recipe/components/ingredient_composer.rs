use crate::components::ui::{
    Button, ButtonVariant, Checkbox, ComboOption, Combobox, Input, Picker, Size, Text,
};
use crate::features::recipe::components::ingredient_line::RecipeLine;
use dioxus::prelude::*;

/// The tinted "add ingredient" well: search-or-create an ingredient, give a
/// brand-new one its unit and pack size, set qty/note, then emit the finished
/// line via `on_add`. The emitted line carries a placeholder `id` — the list
/// owner assigns the real one.
#[component]
pub fn IngredientComposer(
    ingredient_options: Vec<ComboOption>,
    unit_options: Vec<ComboOption>,
    on_add: EventHandler<RecipeLine>,
) -> Element {
    let mut ingredient = use_signal(Picker::default);
    let mut unit = use_signal(Picker::default);
    let mut purchase_size = use_signal(String::new);
    let mut qty = use_signal(String::new);
    let mut note = use_signal(String::new);
    let mut optional = use_signal(|| false);

    let ing = ingredient();
    let creating = ing.is_new();
    let composing = ing.is_resolved();

    let current_unit = if creating {
        let u = unit();
        u.new_name.clone().or(u.picked.map(|o| o.label))
    } else {
        ing.picked.as_ref().map(|o| o.hint.clone())
    };
    let qty_label = match &current_unit {
        Some(u) => format!("Qty ({u})"),
        None => "Qty".to_string(),
    };

    let can_add = if creating {
        unit().is_resolved() && !purchase_size().trim().is_empty() && !qty().trim().is_empty()
    } else {
        ing.picked.is_some() && !qty().trim().is_empty()
    };

    rsx! {
        div { class: "mr-subpanel",
            Combobox {
                label: "Add ingredient".to_string(),
                placeholder: "Search, or type a new one and press Enter…".to_string(),
                options: ingredient_options.clone(),
                picker: ingredient,
                // A fresh create starts the unit/pack-size sub-form clean.
                on_create: move |_| {
                    unit.set(Picker::default());
                    purchase_size.set(String::new());
                },
            }

            if creating {
                Text { muted: true, small: true,
                    "New ingredient — give it a unit and pack size."
                }
                div { class: "mr-form-grid",
                    Combobox {
                        label: "Unit".to_string(),
                        placeholder: "Search, or type + Enter…".to_string(),
                        options: unit_options.clone(),
                        picker: unit,
                    }
                    Input {
                        label: "Purchase size".to_string(),
                        kind: "number".to_string(),
                        value: purchase_size(),
                        oninput: move |e: FormEvent| purchase_size.set(e.value()),
                        placeholder: "e.g. 500".to_string(),
                    }
                }
            }

            if composing {
                div { class: "mr-inline-add",
                    div { class: "mr-qty",
                        Input {
                            label: qty_label.clone(),
                            kind: "number".to_string(),
                            value: qty(),
                            oninput: move |e: FormEvent| qty.set(e.value()),
                        }
                    }
                    Input {
                        label: "Note (optional)".to_string(),
                        value: note(),
                        oninput: move |e: FormEvent| note.set(e.value()),
                        placeholder: "e.g. finely chopped".to_string(),
                    }
                }
                div { class: "mr-inline-add mr-inline-add--split",
                    Checkbox {
                        checked: optional(),
                        onchange: move |v: bool| optional.set(v),
                        label: "Optional".to_string(),
                    }
                    Button {
                        variant: ButtonVariant::Solid,
                        size: Size::Sm,
                        disabled: !can_add,
                        onclick: move |_| {
                            let ing = ingredient();
                            let (name, unit_label, is_new, ingredient_id, unit_id, new_unit) =
                                if let Some(opt) = &ing.picked {
                                    (
                                        opt.label.clone(),
                                        opt.hint.clone(),
                                        false,
                                        Some(opt.id.clone()),
                                        None,
                                        None,
                                    )
                                } else if let Some(name) = &ing.new_name {
                                    let u = unit();
                                    match (&u.new_name, &u.picked) {
                                        (Some(nu), _) => {
                                            (name.clone(), nu.clone(), true, None, None, Some(nu.clone()))
                                        }
                                        (None, Some(po)) => (
                                            name.clone(),
                                            po.label.clone(),
                                            true,
                                            None,
                                            Some(po.id.clone()),
                                            None,
                                        ),
                                        (None, None) => return,
                                    }
                                } else {
                                    return;
                                };
                            on_add.call(RecipeLine {
                                id: 0,
                                name,
                                unit: unit_label,
                                qty: qty().trim().to_string(),
                                note: note().trim().to_string(),
                                optional: optional(),
                                is_new,
                                ingredient_id,
                                purchase_size: purchase_size().trim().to_string(),
                                unit_id,
                                new_unit,
                            });
                            ingredient.set(Picker::default());
                            unit.set(Picker::default());
                            purchase_size.set(String::new());
                            qty.set(String::new());
                            note.set(String::new());
                            optional.set(false);
                        },
                        "Add to recipe"
                    }
                }
            } else {
                Text { muted: true, small: true,
                    "Pick one from the list, or type a new name and press Enter to create it."
                }
            }
        }
    }
}
