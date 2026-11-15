use crate::components::ui::{
    Badge, Button, ButtonVariant, Checkbox, ComboOption, Icon, IconButton, IconName, Input, Picker,
    Size, Tone,
};
use crate::features::recipe::components::import_match::ImportMatch;
use crate::features::recipe::components::ingredient_composer::parse_qty;
use crate::models::{amount_label, qty_label};
use dioxus::prelude::*;
use domain::ingredient::IngredientEntry;
use domain::recipe_import::{ImportedLine, MatchSuggestion};

/// One chosen ingredient in the dialog's in-memory list.
#[derive(Clone, PartialEq)]
pub struct RecipeLine {
    pub id: usize,
    pub name: String,
    pub unit: String,
    pub qty: String,
    pub note: String,
    pub optional: bool,
    pub ingredient: IngredientEntry,
    pub import: Option<ImportChoice>,
}

/// What an imported line can still be switched to: the page's wording, the library ingredient
/// it looks like, and the line each choice gives.
#[derive(Clone, PartialEq)]
pub struct ImportChoice {
    pub phrase: String,
    pub suggestion: Option<MatchSuggestion>,
    pub as_new: ImportedLine,
    pub matched: bool,
}

/// A single row in the "In this recipe" list: a read view with Edit/× actions,
/// or — while `editing` — a small stacked form. The row owns its draft state;
/// `on_save` hands the updated line back to the list owner.
#[component]
pub fn IngredientLine(
    line: RecipeLine,
    editing: bool,
    on_edit: EventHandler<usize>,
    on_cancel: EventHandler<()>,
    on_save: EventHandler<RecipeLine>,
    on_remove: EventHandler<usize>,
    #[props(default)] ingredient_options: Vec<ComboOption>,
    #[props(default)] on_change: Option<EventHandler<(usize, Picker)>>,
    #[props(default)] taken: Vec<String>,
) -> Element {
    let mut draft_qty = use_signal(String::new);
    let mut draft_note = use_signal(String::new);
    let mut draft_optional = use_signal(|| false);

    let id = line.id;

    if editing {
        let base = line.clone();
        rsx! {
            div { class: "mr-ing-line mr-ing-line--editing",
                div { class: "mr-ing-line__head",
                    span { class: "mr-ing-line__name", "{line.name}" }

                    if let IngredientEntry::New { .. } = &line.ingredient {
                        Badge { tone: Tone::Green, "New" }
                    }
                }
                div { class: "mr-inline-add",
                    div { class: "mr-qty",
                        Input {
                            label: qty_label(&line.unit),
                            kind: "number".to_string(),
                            value: draft_qty(),
                            oninput: move |e: FormEvent| draft_qty.set(e.value()),
                            error: parse_qty(&draft_qty()).err(),
                        }
                    }
                    Input {
                        label: "Note".to_string(),
                        value: draft_note(),
                        oninput: move |e: FormEvent| draft_note.set(e.value()),
                    }
                }
                div { class: "mr-inline-add mr-inline-add--split",
                    Checkbox {
                        checked: draft_optional(),
                        onchange: move |v: bool| draft_optional.set(v),
                        label: "Optional".to_string(),
                    }
                    div { class: "mr-ing-line__edit-actions",
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            onclick: move |_| on_cancel.call(()),
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Solid,
                            size: Size::Sm,
                            disabled: parse_qty(&draft_qty()).is_err(),
                            onclick: move |_| {
                                on_save
                                    .call(RecipeLine {
                                        qty: draft_qty().trim().to_string(),
                                        note: draft_note().trim().to_string(),
                                        optional: draft_optional(),
                                        ..base.clone()
                                    });
                            },
                            "Save"
                        }
                    }
                }
            }
        }
    } else {
        let amount = amount_label(parse_qty(&line.qty).ok().flatten(), &line.unit);
        let meta = if line.note.trim().is_empty() {
            amount
        } else {
            format!("{amount} · {}", line.note)
        };
        let e_qty = line.qty.clone();
        let e_note = line.note.clone();
        let e_opt = line.optional;
        rsx! {
            div { class: "mr-ing-line",
                div { class: "mr-ing-line__main",
                    div { class: "mr-ing-line__head",
                        span { class: "mr-ing-line__name", "{line.name}" }
                        if let IngredientEntry::New { .. } = &line.ingredient {
                            Badge { tone: Tone::Green, "New" }
                        } else if line.optional {
                            Badge { tone: Tone::Neutral, "Optional" }
                        }
                    }
                    span { class: "mr-ing-line__meta", "{meta}" }
                    if let Some(on_change) = on_change {
                        ImportMatch {
                            line: line.clone(),
                            ingredient_options: ingredient_options.clone(),
                            on_change,
                            taken: taken.clone(),
                        }
                    }
                }
                div { class: "mr-ing-line__actions",
                    button {
                        r#type: "button",
                        class: "mr-ing-line__edit",
                        aria_label: "Edit ingredient",
                        onclick: move |_| {
                            draft_qty.set(e_qty.clone());
                            draft_note.set(e_note.clone());
                            draft_optional.set(e_opt);
                            on_edit.call(id);
                        },
                        "Edit"
                    }
                    IconButton {
                        label: format!("Remove {}", line.name),
                        danger: true,
                        onclick: move |_| on_remove.call(id),
                        Icon { name: IconName::X }
                    }
                }
            }
        }
    }
}
