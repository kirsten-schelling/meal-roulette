use crate::components::ui::{
    Alert, Button, ButtonVariant, ComboOption, ConfirmDialog, Heading, Icon, IconButton, IconName,
    Modal, Picker, ReasonTip, Separator, Spinner, Text, Tone,
};
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::library::components::{ManageListsDialog, ManageTab};
use crate::features::recipe::components::import_from_link::ImportFromLink;
use crate::features::recipe::components::import_match::ImportMatch;
use crate::features::recipe::components::ingredient_composer::{parse_qty, IngredientComposer};
use crate::features::recipe::components::ingredient_line::{
    ImportChoice, IngredientLine, RecipeLine,
};
use crate::features::recipe::components::recipe_fields::{
    options_from, use_recipe_catalogs, use_recipe_form, RecipeFields,
};
use crate::features::recipe::components::recipe_seed::RecipeSeed;
use crate::features::recipe::hooks::use_recipe::use_add_recipe;
use crate::features::unit::hooks::use_unit::use_get_units;
use crate::hooks::use_toast::use_toast;
use crate::models::{amount_label, error_message};
use dioxus::prelude::*;
use domain::ingredient::IngredientEntry;
use domain::recipe_import::RecipeImport;
use domain::recipe_ingredient::RecipeIngredientEntry;
use domain::unit::UnitEntry;

// Recipe details are home; "Add ingredients" opens the composer. Creating an
// ingredient happens inline in the composer, so there's no extra panel.
#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Recipe,
    Ingredients,
}

/// Units typed as new on earlier lines, offered again (tagged "New") so the next new
/// ingredient can be measured in them before anything is saved.
fn pending_units(lines: &[RecipeLine], saved: &[ComboOption]) -> Vec<ComboOption> {
    let mut pending: Vec<ComboOption> = Vec::new();
    for line in lines {
        let IngredientEntry::New {
            unit: UnitEntry::New(name),
            ..
        } = &line.ingredient
        else {
            continue;
        };
        let known = saved
            .iter()
            .chain(pending.iter())
            .any(|o| o.label.eq_ignore_ascii_case(name));
        if !known {
            pending.push(ComboOption::pending(name.clone()));
        }
    }
    pending
}

fn seeded_lines(seed: &RecipeSeed) -> Vec<RecipeLine> {
    seed.ingredient
        .iter()
        .map(|option| RecipeLine {
            id: 1,
            name: option.label.clone(),
            unit: option.hint.clone(),
            qty: String::new(),
            note: String::new(),
            optional: false,
            ingredient: IngredientEntry::Existing(option.id),
            import: None,
        })
        .collect()
}

// Lines are validated as they are added or edited, so the qty always parses here.
fn line_entries(lines: &[RecipeLine]) -> Vec<RecipeIngredientEntry> {
    lines
        .iter()
        .map(|l| RecipeIngredientEntry {
            ingredient: l.ingredient.clone(),
            qty: parse_qty(&l.qty).ok().flatten(),
            note: (!l.note.is_empty()).then(|| l.note.clone()),
            is_optional: l.optional,
            import_phrase: l.import.as_ref().map(|i| i.phrase.clone()),
        })
        .collect()
}

/// The "Add recipe" dialog. Recipe details are home; a persistent link opens the
/// ingredient composer. Mount it only while open (`if show() { AddRecipeDialog {
/// .. } }`) so each open starts clean; `on_close` fires once the user has closed it
/// (after confirming, if anything was filled in). `seed` starts the form with
/// the cuisine, protein, difficulty and ingredient a roll rule wants.
#[component]
pub fn AddRecipeDialog(
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
    #[props(default)] seed: RecipeSeed,
) -> Element {
    let mut panel = use_signal(|| Panel::Recipe);
    let form = use_recipe_form(None);
    use_hook(|| form.prefill(&seed));
    let catalogs = use_recipe_catalogs();
    let mut units = use_get_units();
    let ingredients = use_get_ingredients();

    // The manage-lists dialog stacks over this one so an in-progress recipe
    // survives a quick rename detour. `None` = closed.
    let mut managing = use_signal(|| None as Option<ManageTab>);
    let mut confirm_discard = use_signal(|| false);

    let mut lines = use_signal(|| seeded_lines(&seed));
    // Hands out unique ids for the in-memory recipe lines (not persisted yet).
    let mut next_id = use_signal(|| lines.peek().len() + 1);
    // Inline edit of an already-added line (one at a time).
    let mut editing_id = use_signal(|| None as Option<usize>);

    // Submit handle — hooks must be created during render, not in the click handler.
    let add_recipe = use_add_recipe();
    let toast = use_toast();
    use_effect(move || {
        if matches!(&*add_recipe.result.read(), Some(Ok(_))) {
            on_saved.call(());
            toast.show(
                Some("Success".to_string()),
                format!("{} added successfully!", form.title.peek().trim()),
                Tone::Green,
            );
        }
    });
    let saving = (add_recipe.loading)();
    let save_error = match &*add_recipe.result.read() {
        Some(Err(e)) => Some(error_message(e)),
        _ => None,
    };

    let ingredient_options = options_from(&ingredients(), |i| {
        ComboOption::new(i.id, i.name.clone()).with_hint(i.unit.clone())
    });
    let saved_units = options_from(&units(), |u| {
        ComboOption::new(u.unit.id, u.unit.name.clone())
    });

    let lines_now = lines();
    let line_count = lines_now.len();
    let mut unit_options = saved_units.clone();
    unit_options.extend(pending_units(&lines_now, &saved_units));
    let taken: Vec<String> = lines_now.iter().map(|l| l.name.to_lowercase()).collect();

    let missing = form.missing();
    let dirty = form.is_dirty() || !lines_now.is_empty();
    let mut request_close = move || {
        if dirty {
            confirm_discard.set(true);
        } else {
            on_close.call(());
        }
    };
    let save = move |_| {
        if let Some(recipe) = form.new_recipe(line_entries(&lines())) {
            add_recipe.submit(recipe);
        }
    };

    // Fills the form from an imported page for the user to review; nothing is saved until
    // they press Save.
    let apply_import = move |recipe: RecipeImport| {
        let mut form = form;
        form.title.set(recipe.title);
        if let Some(mins) = recipe.time_mins {
            form.time_mins.set(mins.to_string());
        }
        if let Some(servings) = recipe.servings {
            form.servings.set(servings.to_string());
        }
        let imported: Vec<RecipeLine> = recipe
            .ingredients
            .into_iter()
            .enumerate()
            .map(|(i, ing)| {
                let matched = ing.use_suggestion;
                let chosen = match (&ing.suggestion, matched) {
                    (Some(s), true) => s.line.clone(),
                    _ => ing.as_new.clone(),
                };
                RecipeLine {
                    id: i + 1,
                    name: chosen.name,
                    unit: chosen.unit,
                    qty: chosen
                        .qty
                        .map(|q| q.normalize().to_string())
                        .unwrap_or_default(),
                    note: chosen.note.unwrap_or_default(),
                    optional: false,
                    ingredient: chosen.ingredient,
                    import: Some(ImportChoice {
                        phrase: ing.phrase,
                        suggestion: ing.suggestion,
                        as_new: ing.as_new,
                        matched,
                    }),
                }
            })
            .collect();
        next_id.set(imported.len() + 1);
        editing_id.set(None);
        lines.set(imported);
    };

    let change_line = move |(id, picked): (usize, Picker)| {
        let mut lines = lines;
        let mut all = lines.write();
        let name = picked.query.trim().to_string();
        if all
            .iter()
            .any(|o| o.id != id && o.name.eq_ignore_ascii_case(&name))
        {
            return;
        }
        let Some(line) = all.iter_mut().find(|l| l.id == id) else {
            return;
        };
        let Some(import) = line.import.as_mut() else {
            return;
        };
        if let Some(opt) = picked.picked {
            if line.ingredient == IngredientEntry::Existing(opt.id) {
                return;
            }
            line.name = opt.label;
            line.unit = opt.hint;
            line.ingredient = IngredientEntry::Existing(opt.id);
            import.matched = import
                .suggestion
                .as_ref()
                .is_some_and(|s| s.line.ingredient == line.ingredient);
        } else if picked.new_name.is_some() {
            if matches!(&line.ingredient, IngredientEntry::New { name: n, .. } if *n == name) {
                return;
            }
            let IngredientEntry::New {
                purchase_size,
                unit,
                ..
            } = import.as_new.ingredient.clone()
            else {
                return;
            };
            line.unit = import.as_new.unit.clone();
            line.ingredient = IngredientEntry::New {
                name: name.clone(),
                purchase_size,
                unit,
            };
            line.name = name;
            import.matched = false;
        }
    };
    let modal_title = match panel() {
        Panel::Recipe => "Add recipe",
        Panel::Ingredients => "Add ingredients",
    };

    rsx! {
        Modal {
            wide: true,
            on_close: move |_| request_close(),
            title: modal_title.to_string(),

            div { class: "mr-recipe-dialog",
                {
                    match panel() {
                        // ── Home: recipe details + a link into the ingredients ──
                        Panel::Recipe => rsx! {
                            div { class: "mr-recipe-dialog__content",
                                RecipeFields {
                                    form,
                                    cuisine_options: catalogs.cuisine_options(),
                                    protein_options: catalogs.protein_options(),
                                    tag_options: catalogs.tag_options(),
                                    on_manage: move |tab: ManageTab| managing.set(Some(tab)),
                                    ImportFromLink {
                                        on_import: apply_import,
                                        has_title: !form.title.read().trim().is_empty(),
                                        on_title: move |text: String| {
                                            let mut form = form;
                                            form.title.set(text);
                                        },
                                    }
                                }

                                if !lines_now.is_empty() {
                                    div { class: "mr-ings-summary",
                                        span { class: "mr-label", "Ingredients ({line_count})" }
                                        div { class: "mr-ings-summary__list",
                                            for line in lines_now.iter() {
                                                {
                                                    let amount = amount_label(parse_qty(&line.qty).ok().flatten(), &line.unit);
                                                    let amt = if line.optional {
                                                        format!("{amount} · optional")
                                                    } else {
                                                        amount
                                                    };
                                                    rsx! {
                                                        div { key: "{line.id}", class: "mr-ings-summary__item",
                                                            div { class: "mr-ings-summary__row",
                                                                if line.import.is_none() {
                                                                    span { class: "mr-ings-summary__name", "{line.name}" }
                                                                }
                                                                span { class: "mr-ings-summary__amt", "{amt}" }
                                                                if line.import.is_some() {
                                                                    IconButton {
                                                                        label: format!("Remove {}", line.name),
                                                                        danger: true,
                                                                        onclick: {
                                                                            let id = line.id;
                                                                            move |_| lines.write().retain(|l| l.id != id)
                                                                        },
                                                                        Icon { name: IconName::X }
                                                                    }
                                                                }
                                                            }
                                                            ImportMatch {
                                                                line: line.clone(),
                                                                ingredient_options: ingredient_options.clone(),
                                                                on_change: change_line,
                                                                taken: taken.clone(),
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Always available — never gated behind the title.
                                button {
                                    r#type: "button",
                                    class: "mr-add-link",
                                    onclick: move |_| panel.set(Panel::Ingredients),
                                    Icon { name: IconName::Plus }
                                    "Add ingredients"
                                }

                                if let Some(msg) = save_error {
                                    Alert { tone: Tone::Red, title: "Couldn't save the recipe", "{msg}" }
                                }
                            }

                            div { class: "mr-dialog-foot",
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: move |_| request_close(),
                                    "Cancel"
                                }
                                ReasonTip { reasons: missing.clone(),
                                    Button {
                                        variant: ButtonVariant::Solid,
                                        disabled: saving || !missing.is_empty(),
                                        onclick: save,
                                        if saving {
                                            Spinner {}
                                        } else {
                                            "Save recipe"
                                        }
                                    }
                                }
                            }
                        },
                        Panel::Ingredients => rsx! {
                            div { class: "mr-recipe-dialog__content",
                                IngredientComposer {
                                    ingredient_options: ingredient_options.clone(),
                                    unit_options: unit_options.clone(),
                                    taken: taken.clone(),
                                    on_manage_units: move |_| managing.set(Some(ManageTab::Units)),
                                    on_add: move |mut line: RecipeLine| {
                                        let id = next_id();
                                        next_id.set(id + 1);
                                        line.id = id;
                                        lines.write().push(line);
                                    },
                                }

                                Separator {}

                                Heading { level: 3, "In this recipe ({line_count})" }

                                if lines_now.is_empty() {
                                    p { class: "mr-empty-note", "Nothing added yet. Use the search above." }
                                } else {
                                    div { class: "mr-ing-list mr-scroll",
                                        for line in lines_now.iter() {
                                            IngredientLine {
                                                key: "{line.id}",
                                                line: line.clone(),
                                                editing: editing_id() == Some(line.id),
                                                ingredient_options: ingredient_options.clone(),
                                                on_change: change_line,
                                                taken: taken.clone(),
                                                on_edit: move |id: usize| editing_id.set(Some(id)),
                                                on_cancel: move |_| editing_id.set(None),
                                                on_save: move |updated: RecipeLine| {
                                                    if let Some(l) =
                                                        lines.write().iter_mut().find(|l| l.id == updated.id)
                                                    {
                                                        *l = updated;
                                                    }
                                                    editing_id.set(None);
                                                },
                                                on_remove: move |id: usize| {
                                                    lines.write().retain(|l| l.id != id);
                                                },
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "mr-dialog-foot",
                                Button {
                                    variant: ButtonVariant::Solid,
                                    onclick: move |_| panel.set(Panel::Recipe),
                                    Icon { name: IconName::ArrowLeft }
                                    "Back to recipe"
                                }
                            }
                        },
                    }
                }
            }
        }

        if let Some(tab) = managing() {
            ManageListsDialog {
                initial_tab: tab,
                on_close: move |_| {
                    managing.set(None);
                    // Renames may have landed while it was open — refresh the
                    // catalogs feeding the comboboxes.
                    catalogs.restart();
                    units.restart();
                },
            }
        }

        if confirm_discard() {
            ConfirmDialog {
                title: "Discard this recipe?".to_string(),
                confirm_label: "Discard".to_string(),
                danger: true,
                on_confirm: move |_| on_close.call(()),
                on_cancel: move |_| confirm_discard.set(false),
                Text { "You've started filling it in. Closing now loses what you've entered." }
            }
        }
    }
}
