use crate::components::ui::{
    Button, ButtonVariant, Checkbox, ComboOption, Combobox, Input, Picker, Size, Text,
};
use crate::features::ingredient::parse::parse_purchase_size;
use crate::features::recipe::components::ingredient_line::RecipeLine;
use crate::models::qty_label;
use dioxus::prelude::*;
use domain::ingredient::IngredientEntry;
use domain::unit::UnitEntry;
use rust_decimal::Decimal;

/// A quantity as typed: `Ok(None)` when left blank, `Ok(Some)` once it is a number
/// above zero.
pub fn parse_qty(value: &str) -> Result<Option<Decimal>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    match value.parse::<Decimal>() {
        Ok(qty) if qty > Decimal::ZERO => Ok(Some(qty)),
        Ok(_) => Err("Qty must be more than 0".to_string()),
        Err(_) => Err("Qty must be a number".to_string()),
    }
}

/// The tinted "add ingredient" well: search-or-create an ingredient, give a
/// brand-new one its unit and pack size, set qty/note, then emit the finished
/// line via `on_add`. The emitted line carries a placeholder `id` — the list
/// owner assigns the real one. `taken` holds the lowercased names already in
/// the recipe, so the same ingredient can't be added twice.
#[component]
pub fn IngredientComposer(
    ingredient_options: Vec<ComboOption>,
    unit_options: Vec<ComboOption>,
    on_add: EventHandler<RecipeLine>,
    on_manage_units: Option<EventHandler<()>>,
    #[props(default)] taken: Vec<String>,
    #[props(default)] busy: bool,
) -> Element {
    let mut ingredient = use_signal(Picker::default);
    let mut unit = use_signal(Picker::default);
    let mut purchase_size = use_signal(String::new);
    let mut qty = use_signal(String::new);
    let mut note = use_signal(String::new);
    let mut optional = use_signal(|| false);
    // Errors show once a field has been typed in, not while it's still untouched.
    let mut qty_touched = use_signal(|| false);
    let mut size_touched = use_signal(|| false);

    let ing = ingredient();
    let creating = ing.is_new();
    let composing = ing.is_resolved();

    let duplicate = ing.is_resolved() && taken.contains(&ing.query.trim().to_lowercase());

    let current_unit = if creating {
        let u = unit();
        u.new_name.clone().or(u.picked.map(|o| o.label))
    } else {
        ing.picked.as_ref().map(|o| o.hint.clone())
    }
    .filter(|u| !u.is_empty());
    let qty_label = qty_label(current_unit.as_deref().unwrap_or_default());

    let qty_check = parse_qty(&qty());
    let size_check = parse_purchase_size(&purchase_size());
    let can_add = !busy
        && !duplicate
        && qty_check.is_ok()
        && if creating {
            (unit().is_resolved() || unit().query.trim().is_empty()) && size_check.is_ok()
        } else {
            ing.picked.is_some()
        };
    let qty_error = qty_touched().then(|| qty_check.clone().err()).flatten();
    let size_error = size_touched().then(|| size_check.clone().err()).flatten();

    let mut reset = move || {
        ingredient.set(Picker::default());
        unit.set(Picker::default());
        purchase_size.set(String::new());
        qty.set(String::new());
        note.set(String::new());
        optional.set(false);
        qty_touched.set(false);
        size_touched.set(false);
    };

    let submit = move |_| {
        let ing = ingredient();
        let (name, unit_label, entry) = match (&ing.picked, &ing.new_name) {
            (Some(opt), _) => (
                opt.label.clone(),
                opt.hint.clone(),
                IngredientEntry::Existing(opt.id),
            ),
            (None, Some(name)) => {
                let u = unit();
                let (unit_label, unit_entry) = match (&u.new_name, &u.picked) {
                    (Some(new_unit), _) => {
                        (new_unit.clone(), Some(UnitEntry::New(new_unit.clone())))
                    }
                    (None, Some(picked)) => {
                        (picked.label.clone(), Some(UnitEntry::Existing(picked.id)))
                    }
                    (None, None) if u.query.trim().is_empty() => (String::new(), None),
                    (None, None) => return,
                };
                let Ok(size) = parse_purchase_size(&purchase_size()) else {
                    return;
                };
                (
                    name.clone(),
                    unit_label,
                    IngredientEntry::New {
                        name: name.clone(),
                        purchase_size: size,
                        unit: unit_entry,
                    },
                )
            }
            (None, None) => return,
        };
        on_add.call(RecipeLine {
            id: 0,
            name,
            unit: unit_label,
            qty: qty().trim().to_string(),
            note: note().trim().to_string(),
            optional: optional(),
            ingredient: entry,
            import: None,
        });
        reset();
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
                    size_touched.set(false);
                },
            }

            if duplicate {
                Text { small: true, "That ingredient is already in this recipe. Edit its line instead." }
            }

            if creating && !duplicate {
                Text { muted: true, small: true, "New ingredient: give it a unit and a pack size if it has them." }
                div { class: "mr-form-grid",
                    Combobox {
                        label: "Unit (optional)".to_string(),
                        placeholder: "Search, or type + Enter…".to_string(),
                        options: unit_options.clone(),
                        picker: unit,
                        manage_label: "Manage units…".to_string(),
                        on_manage: on_manage_units,
                    }
                    Input {
                        label: "Purchase size (optional)".to_string(),
                        kind: "number".to_string(),
                        value: purchase_size(),
                        oninput: move |e: FormEvent| {
                            purchase_size.set(e.value());
                            size_touched.set(true);
                        },
                        onblur: move |_| size_touched.set(true),
                        error: size_error,
                        placeholder: "e.g. 500".to_string(),
                    }
                }
            }

            if composing && !duplicate {
                div { class: "mr-inline-add",
                    div { class: "mr-qty",
                        Input {
                            label: qty_label.clone(),
                            kind: "number".to_string(),
                            value: qty(),
                            placeholder: "Optional".to_string(),
                            oninput: move |e: FormEvent| {
                                qty.set(e.value());
                                qty_touched.set(true);
                            },
                            onblur: move |_| qty_touched.set(true),
                            error: qty_error,
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
                        loading: busy,
                        onclick: submit,
                        "Add to recipe"
                    }
                }
            } else if !composing {
                Text { muted: true, small: true,
                    "Pick one from the list, or type a new name and press Enter to create it."
                }
            }
        }
    }
}
