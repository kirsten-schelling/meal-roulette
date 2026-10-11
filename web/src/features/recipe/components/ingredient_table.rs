use crate::components::ui::{
    Badge, CellAlign, Checkbox, Column, ConfirmDialog, DataTable, Icon, IconButton, IconName,
    Input, Text, Tone,
};
use crate::features::recipe::components::ingredient_composer::parse_qty;
use crate::features::recipe::hooks::use_recipe::{
    remove_recipe_ingredient, update_recipe_ingredient,
};
use crate::hooks::use_toast::use_toast;
use crate::models::{amount_label, error_message};
use dioxus::prelude::*;
use domain::recipe_ingredient::{RecipeIngredient, UpdateRecipeIngredient};

/// The line being edited, as typed so far.
#[derive(Clone, PartialEq)]
struct LineDraft {
    line: RecipeIngredient,
    qty: String,
    note: String,
    optional: bool,
}

impl LineDraft {
    fn new(line: &RecipeIngredient) -> Self {
        LineDraft {
            line: line.clone(),
            qty: line
                .qty
                .map(|q| q.normalize().to_string())
                .unwrap_or_default(),
            note: line.note.clone().unwrap_or_default(),
            optional: line.is_optional,
        }
    }

    fn is(&self, line: &RecipeIngredient) -> bool {
        self.line.ingredient_id == line.ingredient_id
    }

    /// `None` when nothing changed; `Err` when the quantity does not parse.
    fn update(&self) -> Result<Option<UpdateRecipeIngredient>, String> {
        let qty = parse_qty(&self.qty)?;
        let trimmed = self.note.trim();
        let note = (!trimmed.is_empty()).then(|| trimmed.to_string());
        if qty == self.line.qty && note == self.line.note && self.optional == self.line.is_optional
        {
            return Ok(None);
        }
        Ok(Some(UpdateRecipeIngredient {
            qty,
            note,
            is_optional: self.optional,
        }))
    }
}

/// A saved recipe's ingredient lines. Clicking a line opens its amount, note and
/// "optional" flag for editing in the row itself; one line is open at a time.
/// `on_changed` fires after a line is saved or removed.
#[component]
pub fn IngredientTable(lines: Vec<RecipeIngredient>, on_changed: EventHandler<()>) -> Element {
    let toast = use_toast();
    let mut draft = use_signal(|| None as Option<LineDraft>);
    let mut saving = use_signal(|| false);
    let mut removing = use_signal(|| None as Option<RecipeIngredient>);

    // Another line may take over only while the open one holds no unsaved typing.
    let mut start = move |line: &RecipeIngredient| {
        let untouched = draft
            .read()
            .as_ref()
            .is_none_or(|d| matches!(d.update(), Ok(None)));
        if untouched {
            draft.set(Some(LineDraft::new(line)));
        }
    };
    let mut cancel = move || draft.set(None);
    let mut save = move || {
        let Some(current) = draft() else {
            return;
        };
        if saving() {
            return;
        }
        let update = match current.update() {
            Ok(Some(update)) => update,
            Ok(None) => {
                draft.set(None);
                return;
            }
            Err(_) => return,
        };
        let line = current.line;
        saving.set(true);
        spawn(async move {
            match update_recipe_ingredient(line.recipe_id, line.ingredient_id, update).await {
                Ok(_) => {
                    draft.set(None);
                    on_changed.call(());
                }
                Err(e) => toast.show(
                    Some(format!("Couldn't save {}", line.ingredient_name)),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            saving.set(false);
        });
    };
    let remove = move |line: RecipeIngredient| {
        spawn(async move {
            match remove_recipe_ingredient(line.recipe_id, line.ingredient_id).await {
                Ok(_) => {
                    on_changed.call(());
                    toast.show(
                        Some("Removed".to_string()),
                        format!("{} is no longer in the recipe.", line.ingredient_name),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't remove it".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            removing.set(None);
        });
    };

    let keys = move |e: KeyboardEvent| {
        e.stop_propagation();
        match e.key() {
            Key::Enter => save(),
            Key::Escape => cancel(),
            _ => {}
        }
    };
    let editing = move |l: &RecipeIngredient| draft.read().as_ref().filter(|d| d.is(l)).cloned();

    let columns: Vec<Column<RecipeIngredient>> = vec![
        Column::new("Ingredient", move |l: &RecipeIngredient| match editing(l) {
            Some(d) => rsx! {
                div { class: "mr-line-edit__name",
                    span { class: "mr-cell-strong", "{l.ingredient_name}" }
                    Checkbox {
                        checked: d.optional,
                        onchange: move |v: bool| {
                            if let Some(d) = draft.write().as_mut() {
                                d.optional = v;
                            }
                        },
                        label: "Optional".to_string(),
                    }
                }
            },
            None => rsx! {
                span { class: "mr-cell-strong", "{l.ingredient_name}" }
                if l.is_optional {
                    " "
                    Badge { "Optional" }
                }
            },
        }),
        Column::new("Amount", move |l: &RecipeIngredient| match editing(l) {
            Some(d) => rsx! {
                div { class: "mr-line-edit__qty", onkeydown: keys,
                    Input {
                        kind: "number".to_string(),
                        value: d.qty.clone(),
                        autofocus: true,
                        placeholder: "Optional".to_string(),
                        oninput: move |e: FormEvent| {
                            if let Some(d) = draft.write().as_mut() {
                                d.qty = e.value();
                            }
                        },
                        error: parse_qty(&d.qty).err(),
                    }
                    span { class: "mr-line-edit__unit", "{l.unit}" }
                }
            },
            None => rsx! { "{amount_label(l.qty, &l.unit)}" },
        })
        .align(CellAlign::End),
        Column::new("Note", move |l: &RecipeIngredient| match editing(l) {
            Some(d) => rsx! {
                div { onkeydown: keys,
                    Input {
                        value: d.note.clone(),
                        oninput: move |e: FormEvent| {
                            if let Some(d) = draft.write().as_mut() {
                                d.note = e.value();
                            }
                        },
                        placeholder: "e.g. finely chopped".to_string(),
                    }
                }
            },
            None => match &l.note {
                Some(note) => rsx! { "{note}" },
                None => rsx! {
                    span { class: "mr-text--muted", "None" }
                },
            },
        }),
        Column::new("", move |l: &RecipeIngredient| {
            if editing(l).is_some() {
                return rsx! {
                    div { class: "mr-rowactions",
                        IconButton {
                            label: format!("Save {}", l.ingredient_name),
                            onclick: move |_| save(),
                            Icon { name: IconName::Check }
                        }
                        IconButton {
                            label: "Cancel".to_string(),
                            onclick: move |_| cancel(),
                            Icon { name: IconName::X }
                        }
                    }
                };
            }
            let to_edit = l.clone();
            let to_remove = l.clone();
            rsx! {
                div { class: "mr-rowactions",
                    IconButton {
                        label: format!("Edit {}", l.ingredient_name),
                        onclick: move |_| start(&to_edit),
                        Icon { name: IconName::Edit }
                    }
                    IconButton {
                        label: format!("Remove {}", l.ingredient_name),
                        danger: true,
                        onclick: move |_| removing.set(Some(to_remove.clone())),
                        Icon { name: IconName::X }
                    }
                }
            }
        })
        .align(CellAlign::End)
        .interactive(),
    ];

    let rows = lines.clone();
    rsx! {
        DataTable::<RecipeIngredient> {
            columns,
            rows: lines,
            empty: "No ingredients yet. Add the first one below.".to_string(),
            on_row_click: move |i: usize| {
                if let Some(line) = rows.get(i) {
                    start(line);
                }
            },
        }

        if let Some(line) = removing() {
            ConfirmDialog {
                title: format!("Remove {}?", line.ingredient_name),
                confirm_label: "Remove".to_string(),
                danger: true,
                on_confirm: move |_| remove(line.clone()),
                on_cancel: move |_| removing.set(None),
                Text { "It comes off this recipe only; the ingredient stays in your library." }
            }
        }
    }
}
