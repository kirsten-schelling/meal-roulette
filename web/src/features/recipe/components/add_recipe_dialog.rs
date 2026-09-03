use crate::components::ui::{
    Button, ButtonVariant, ComboOption, Combobox, Heading, Input, Modal, Picker, Rating,
    RatingScale, Separator, Spinner, Tone,
};
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::library::components::{ManageListsDialog, ManageTab};
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::recipe::components::ingredient_composer::IngredientComposer;
use crate::features::recipe::components::ingredient_line::{IngredientLine, RecipeLine};
use crate::features::recipe::hooks::use_recipe::use_add_recipe;
use crate::features::unit::hooks::use_unit::use_get_units;
use crate::hooks::use_toast::{use_toast, ToastController};
use dioxus::prelude::*;
use domain::cuisine::CuisineEntry;
use domain::protein::ProteinEntry;
use domain::recipe::NewRecipe;
use domain::recipe_ingredient::RecipeIngredientEntry;

// Recipe details are home; "Add ingredients" opens the composer. Creating an
// ingredient happens inline in the composer, so there's no extra panel.
#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Recipe,
    Ingredients,
}

#[derive(PartialEq)]
struct NewRecipeForm {
    title: String,
    difficulty: i16,
    time_mins: String,
    servings: String,
    healthiness: Option<i16>,
    cuisine: Picker,
    protein: Picker,
    lines: Vec<RecipeLine>,
}

/// Flattens a lookup fetch into combobox rows — empty while loading or on error.
fn options_from<T>(
    fetched: &Option<Result<Vec<T>, ServerFnError>>,
    to_option: impl Fn(&T) -> ComboOption,
) -> Vec<ComboOption> {
    match fetched {
        Some(Ok(items)) => items.iter().map(to_option).collect(),
        _ => Vec::new(),
    }
}

fn validate_form(
    title: String,
    difficulty: i16,
    time_mins: Signal<String, UnsyncStorage>,
    servings: Signal<String, UnsyncStorage>,
    protein: Picker,
    cuisine: Picker,
) -> bool {
    if title.trim().is_empty()
        || number_field_error(&time_mins(), "Time").is_some()
        || number_field_error(&servings(), "Servings").is_some()
        || !(1..=5).contains(&difficulty)
        || !protein.is_resolved()
        || !cuisine.is_resolved()
    {
        return false;
    }
    true
}

fn number_field_error(value: &str, name: &str) -> Option<String> {
    if value.is_empty() {
        return Some(format!("{} is required", name));
    } else if value.parse::<i16>().is_err() {
        return Some(format!("{} is invalid", name));
    }
    None
}

/// The "Add recipe" dialog. Recipe details are home; a persistent link opens the
/// ingredient composer. Mount it only while open (`if show() { AddRecipeDialog {
/// .. } }`) so each open starts clean; `on_close` fires on the ×, the backdrop, or
/// Cancel.
#[component]
pub fn AddRecipeDialog(on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let mut panel = use_signal(|| Panel::Recipe);

    let mut title = use_signal(String::new);
    let mut difficulty = use_signal(|| 3i16);
    let mut time_mins = use_signal(String::new);
    let mut servings = use_signal(|| "2".to_string());
    let mut healthiness = use_signal(|| None as Option<i16>);

    // Lookup catalogs come from the API; each field's Picker tracks the typed
    // text plus the resolved choice (existing id, or a name to create at submit).
    let mut cuisines = use_get_cuisines();
    let cuisine = use_signal(Picker::default);
    let mut proteins = use_get_proteins();
    let protein = use_signal(Picker::default);
    let mut units = use_get_units();
    let ingredients = use_get_ingredients();

    let mut time_mins_error = use_signal(|| None as Option<String>);
    let mut servings_error = use_signal(|| None as Option<String>);
    // The manage-lists dialog stacks over this one so an in-progress recipe
    // survives a quick rename detour. `None` = closed.
    let mut managing = use_signal(|| None as Option<ManageTab>);

    let mut lines = use_signal(Vec::<RecipeLine>::new);
    // Hands out unique ids for the in-memory recipe lines (not persisted yet).
    let mut next_id = use_signal(|| 1usize);
    // Inline edit of an already-added line (one at a time).
    let mut editing_id = use_signal(|| None as Option<usize>);

    // Submit handle — hooks must be created during render, not in the click handler.
    let add_recipe = use_add_recipe();
    let use_toast = use_toast();
    // On a successful save the page closes us and refetches; an error leaves the
    // dialog open for now (a toast will come later).
    use_effect(move || {
        if matches!(&*add_recipe.result.read(), Some(Ok(_))) {
            on_saved.call(());
            use_toast.show(
                Some("Success".to_string()),
                format!("{} added successfully!", title()),
                Tone::Green,
            );
        }
    });
    let saving = (add_recipe.loading)();

    let cuisine_options = options_from(&cuisines(), |c| ComboOption::new(c.id, c.name.clone()));
    let protein_options = options_from(&proteins(), |p| ComboOption::new(p.id, p.name.clone()));
    let ingredient_options = options_from(&ingredients(), |i| {
        ComboOption::new(i.id, i.name.clone()).with_hint(i.unit.clone())
    });
    let unit_options = options_from(&units(), |u| ComboOption::new(u.id, u.name.clone()));

    let lines_now = lines();
    let line_count = lines_now.len();

    let modal_title = match panel() {
        Panel::Recipe => "Add recipe",
        Panel::Ingredients => "Add ingredients",
    };

    rsx! {
        Modal {
            wide: true,
            overflow_visible: true,
            on_close: move |_| on_close.call(()),
            title: modal_title.to_string(),

            div { class: "mr-recipe-dialog",
                {
                    match panel() {
                        // ── Home: recipe details + a link into the ingredients ──
                        Panel::Recipe => rsx! {
                            div { class: "mr-recipe-dialog__content",
                                Input {
                                    label: "Title".to_string(),
                                    value: title(),
                                    oninput: move |e: FormEvent| title.set(e.value()),
                                    placeholder: "e.g. Pad Thai".to_string(),
                                }
                                div { class: "mr-form-grid",
                                    Combobox {
                                        label: "Main protein".to_string(),
                                        placeholder: "Search, or type a new one and press Enter…".to_string(),
                                        options: protein_options.clone(),
                                        picker: protein,
                                        manage_label: "Manage proteins…".to_string(),
                                        on_manage: move |_| managing.set(Some(ManageTab::Proteins)),
                                    }
                                    Combobox {
                                        label: "Cuisine".to_string(),
                                        placeholder: "Search, or type a new one and press Enter…".to_string(),
                                        options: cuisine_options.clone(),
                                        picker: cuisine,
                                        manage_label: "Manage cuisines…".to_string(),
                                        on_manage: move |_| managing.set(Some(ManageTab::Cuisines)),
                                    }
                                    Rating {
                                        label: "Difficulty".to_string(),
                                        value: Some(difficulty()),
                                        scale: RatingScale::HighIsBad,
                                        onchange: move |n: i16| difficulty.set(n),
                                    }
                                    Rating {
                                        label: "Healthiness".to_string(),
                                        value: healthiness(),
                                        scale: RatingScale::HighIsGood,
                                        onchange: move |n: i16| healthiness.set(Some(n)),
                                        on_clear: move |_| healthiness.set(None),
                                    }
                                    Input {
                                        label: "Time (mins)".to_string(),
                                        kind: "number".to_string(),
                                        value: time_mins(),
                                        oninput: move |e: FormEvent| {
                                            time_mins.set(e.value());
                                            time_mins_error.set(number_field_error(&time_mins(), "Time"));
                                        },
                                        placeholder: "e.g. 25".to_string(),
                                        error: time_mins_error(),
                                        onblur: move |_| {
                                            time_mins_error.set(number_field_error(&time_mins(), "Time"));
                                        },
                                    }
                                    Input {
                                        label: "Servings".to_string(),
                                        kind: "number".to_string(),
                                        value: servings(),
                                        oninput: move |e: FormEvent| {
                                            servings.set(e.value());
                                            servings_error.set(number_field_error(&servings(), "Servings"));
                                        },
                                        error: servings_error(),
                                        onblur: move |_| {
                                            servings_error.set(number_field_error(&servings(), "Servings"));
                                        },
                                    }
                                }

                                if !lines_now.is_empty() {
                                    div { class: "mr-ings-summary",
                                        span { class: "mr-label", "Ingredients ({line_count})" }
                                        div { class: "mr-ings-summary__list mr-scroll",
                                            for line in lines_now.iter() {
                                                {
                                                    let amt = if line.optional {
                                                        format!("{} {} · optional", line.qty, line.unit)
                                                    } else {
                                                        format!("{} {}", line.qty, line.unit)
                                                    };
                                                    rsx! {
                                                        div { key: "{line.id}", class: "mr-ings-summary__row",
                                                            span { class: "mr-ings-summary__name", "{line.name}" }
                                                            span { class: "mr-ings-summary__amt", "{amt}" }
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
                                    "＋ Add ingredients"
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
                                    disabled: saving
                                        || !validate_form(
                                            title(),
                                            difficulty(),
                                            time_mins,
                                            servings,
                                            protein(),
                                            cuisine(),
                                        ),
                                    onclick: move |_| {
                                        add_recipe
                                            .submit(
                                                build_new_recipe(NewRecipeForm {
                                                    title: title(),
                                                    difficulty: difficulty(),
                                                    time_mins: time_mins(),
                                                    servings: servings(),
                                                    healthiness: healthiness(),
                                                    cuisine: cuisine(),
                                                    protein: protein(),
                                                    lines: lines(),
                                                }),
                                            );
                                    },
                                    if saving {
                                        Spinner {}
                                    } else {
                                        "Save recipe"
                                    }
                                }
                            }
                        },
                        Panel::Ingredients => rsx! {
                            div { class: "mr-recipe-dialog__content",
                                IngredientComposer {
                                    ingredient_options: ingredient_options.clone(),
                                    unit_options: unit_options.clone(),
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
                                    "← Back to recipe"
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
                    cuisines.restart();
                    proteins.restart();
                    units.restart();
                },
            }
        }
    }
}

// Text inputs are parsed to their real types here, at the edge, so the wire
// type carries Decimal/i16 the rest of the way.
fn build_new_recipe(form: NewRecipeForm) -> NewRecipe {
    let ingredients: Vec<RecipeIngredientEntry> = form
        .lines
        .iter()
        .map(|l| RecipeIngredientEntry {
            ingredient: l.ingredient.clone(),
            qty: l.qty.parse().unwrap_or_default(),
            note: if l.note.is_empty() {
                None
            } else {
                Some(l.note.clone())
            },
            is_optional: l.optional,
        })
        .collect();

    let cuisine = form
        .cuisine
        .resolve(CuisineEntry::Existing, CuisineEntry::New);
    let protein = form
        .protein
        .resolve(ProteinEntry::Existing, ProteinEntry::New);

    NewRecipe {
        household_id: None,
        protein,
        cuisine,
        ingredients,
        title: form.title,
        difficulty: form.difficulty,
        time_mins: form.time_mins.parse().unwrap(),
        servings: form.servings.parse().unwrap(),
        healthiness: form.healthiness,
    }
}
