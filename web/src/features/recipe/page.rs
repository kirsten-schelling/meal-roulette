use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Alert, Badge, Button, ButtonVariant, CellAlign, Column, ConfirmDialog, DataTable, FilterField,
    Heading, Icon, IconButton, IconName, SearchBar, Size, Spinner, Text, Tone,
};
use crate::features::recipe::components::{AddRecipeDialog, RecipeCard, ViewToggle};
use crate::features::recipe::filter::{
    matching, CUISINE, DIFFICULTY, HEALTHINESS, MAX_TIME, PROTEIN, SERVINGS, TAG,
};
use crate::features::recipe::hooks::use_recipe::{delete_recipe, use_get_recipe_overviews};
use crate::features::recipe::hooks::use_recipe_view::{use_recipe_view, RecipeView};
use crate::features::roll::settings::DEFAULT_NUM_DAYS;
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use crate::router::Route;
use dioxus::prelude::*;
use domain::recipe::{Recipe, RecipeOverview};

/// Recipes needed to roll a first week: one per day of the roll page's default.
const FIRST_WEEK_RECIPES: usize = DEFAULT_NUM_DAYS;

/// How many tags a table row shows before summarising the rest as "+N".
const TAGS_SHOWN: usize = 3;

pub fn healthiness_badge(healthiness: Option<i16>) -> Element {
    match healthiness {
        Some(h) => {
            let tone = match h {
                1 | 2 => Tone::Red,
                3 => Tone::Amber,
                _ => Tone::Green,
            };
            rsx! {
                Badge { tone, "{h}/5" }
            }
        }
        None => rsx! {
            span { class: "mr-text--muted", "Not rated" }
        },
    }
}

fn muted_or(value: &Option<String>) -> Element {
    match value {
        Some(v) => rsx! { "{v}" },
        None => rsx! {
            span { class: "mr-text--muted", "None" }
        },
    }
}

#[component]
pub fn Recipes() -> Element {
    let mut recipes = use_get_recipe_overviews();
    let mut adding = use_signal(|| false);
    let mut query = use_signal(String::new);
    let mut chips = use_signal(Vec::<String>::new);
    let mut selected_rows = use_signal(Vec::<usize>::new);
    let toast = use_toast();
    let navigator = use_navigator();
    // The recipes waiting on the delete confirmation; empty = no dialog.
    let mut deleting = use_signal(Vec::<Recipe>::new);
    let mut delete_busy = use_signal(|| false);
    let mut search_reset = use_signal(|| 0u32);
    let mut view = use_recipe_view();

    // The rows actually on screen; selection indexes and row clicks refer to these.
    let visible = use_memo(move || match &*recipes.read() {
        Some(Ok(list)) => matching(list, &query(), &chips()),
        _ => Vec::new(),
    });

    let mut confirm_selected = move || {
        let shown = visible.read();
        let chosen: Vec<Recipe> = selected_rows()
            .iter()
            .filter_map(|&i| shown.get(i).map(|o| o.recipe.clone()))
            .collect();
        deleting.set(chosen);
    };

    let open_row = move |i: usize| {
        if let Some(found) = visible.read().get(i) {
            navigator.push(Route::RecipePage {
                id: found.recipe.id,
            });
        }
    };

    let mut delete_confirmed = move || {
        if delete_busy() {
            return;
        }
        delete_busy.set(true);
        spawn(async move {
            let targets = deleting();
            let mut refused = Vec::new();
            for recipe in &targets {
                if let Err(e) = delete_recipe(recipe.id).await {
                    refused.push(format!("{}: {}", recipe.title, error_message(&e)));
                }
            }
            let deleted = targets.len() - refused.len();
            if deleted > 0 {
                recipes.restart();
            }
            if refused.is_empty() {
                let message = match targets.as_slice() {
                    [only] => format!("“{}” was deleted.", only.title),
                    _ => format!("{deleted} recipes were deleted."),
                };
                toast.show(Some("Deleted".to_string()), message, Tone::Green);
            } else {
                let mut message = refused.join(" ");
                if deleted > 0 {
                    message = format!("Deleted {deleted}. {message}");
                }
                toast.show(Some("Couldn't delete".to_string()), message, Tone::Red);
            }
            deleting.set(Vec::new());
            delete_busy.set(false);
        });
    };

    // The table's shape: each column says how to render its own cell from a
    // `&RecipeOverview`. Nothing recipe-specific lives inside `DataTable` itself.
    let columns: Vec<Column<RecipeOverview>> = vec![
        Column::new("Title", |r: &RecipeOverview| {
            rsx! {
                span { class: "mr-cell-strong", "{r.recipe.title}" }
            }
        }),
        Column::new("Cuisine", |r: &RecipeOverview| muted_or(&r.cuisine)),
        Column::new("Protein", |r: &RecipeOverview| muted_or(&r.protein)),
        Column::new("Tags", |r: &RecipeOverview| {
            let extra = r.tags.len().saturating_sub(TAGS_SHOWN);
            rsx! {
                div { class: "mr-tagcell",
                    for tag in r.tags.iter().take(TAGS_SHOWN) {
                        Badge { key: "{tag.id}", "{tag.name}" }
                    }
                    if extra > 0 {
                        span { class: "mr-text--muted", "+{extra}" }
                    }
                }
            }
        }),
        Column::new("Difficulty", |r: &RecipeOverview| {
            rsx! {
                DifficultyBadge { difficulty: r.recipe.difficulty }
            }
        }),
        Column::new("Healthiness", |r: &RecipeOverview| {
            healthiness_badge(r.recipe.healthiness)
        }),
        Column::new(
            "Time",
            |r: &RecipeOverview| rsx! { "{r.recipe.time_mins} min" },
        )
        .align(CellAlign::End),
        Column::new(
            "Servings",
            |r: &RecipeOverview| rsx! { "{r.recipe.servings}" },
        )
        .align(CellAlign::End),
        Column::new("", move |r: &RecipeOverview| {
            let recipe = r.recipe.clone();
            rsx! {
                div { class: "mr-rowactions",
                    IconButton {
                        label: "Delete recipe".to_string(),
                        danger: true,
                        onclick: move |_| deleting.set(vec![recipe.clone()]),
                        Icon { name: IconName::X }
                    }
                }
            }
        })
        .align(CellAlign::End)
        .interactive(),
    ];

    let filtering = !query().trim().is_empty() || !chips().is_empty();
    let empty = "No recipes yet. Add your first one.";
    let total = match &*recipes.read() {
        Some(Ok(list)) => list.len(),
        _ => 0,
    };
    let no_matches = filtering && total > 0 && visible.read().is_empty();
    let filter_summary = {
        let mut parts = Vec::new();
        if !query().trim().is_empty() {
            parts.push(format!("“{}”", query().trim()));
        }
        parts.extend(chips());
        parts.join(", ")
    };

    rsx! {
        div { class: "mr-page",
            // ── Header: titles on the left, the add button on the right.
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "Recipes" }
                    Text { muted: true, "Your recipe library: everything the roulette can pick from." }
                }
                Button {
                    variant: ButtonVariant::Solid,
                    onclick: move |_| adding.set(true),
                    Icon { name: IconName::Plus }
                    "Add recipe"
                }
            }

            if matches!(&*recipes.read(), Some(Ok(_))) && total < FIRST_WEEK_RECIPES {
                div { class: "mr-guide",
                    div { class: "mr-guide__body",
                        Heading { level: 3, "Add {FIRST_WEEK_RECIPES} recipes to roll your first week" }
                        Text { muted: true,
                            "The roulette picks each dinner from your library, so it needs enough recipes to fill the week."
                        }
                        div { class: "mr-guide__progress",
                            span {
                                class: "mr-guide__track",
                                role: "progressbar",
                                aria_label: "Recipes added",
                                aria_valuemin: "0",
                                aria_valuemax: "{FIRST_WEEK_RECIPES}",
                                aria_valuenow: "{total}",
                                span {
                                    class: "mr-guide__fill",
                                    style: "width:{total * 100 / FIRST_WEEK_RECIPES}%",
                                }
                            }
                            span { class: "mr-guide__count", "{total} of {FIRST_WEEK_RECIPES}" }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Solid,
                        onclick: move |_| adding.set(true),
                        Icon { name: IconName::Plus }
                        "Add recipe"
                    }
                }
            }

            if adding() {
                AddRecipeDialog {
                    on_close: move |_| adding.set(false),
                    on_saved: move |_| {
                        recipes.restart();
                        adding.set(false);
                    },
                }
            }

            // ── Combined search + filter bar. Free text plus filter chips added
            //    from the "+" menu. The generic SearchBar owns the interaction;
            //    the page supplies the recipe-specific filter types and applies them.
            div { class: "mr-page__filters",
                div { class: "mr-page__searchrow",
                SearchBar {
                    placeholder: "Search recipes…",
                    filters: vec![
                        FilterField::new(TAG, "e.g. Vegetarian"),
                        FilterField::new(CUISINE, "e.g. Thai"),
                        FilterField::new(PROTEIN, "e.g. Chicken"),
                        FilterField::new(DIFFICULTY, "1–5, or a range like 2-4"),
                        FilterField::new(MAX_TIME, "minutes"),
                        FilterField::new(SERVINGS, "e.g. 4"),
                        FilterField::new(HEALTHINESS, "1–5, or a range like 3-5"),
                    ],
                    onquery: move |q: String| query.set(q),
                    onfilters: move |list: Vec<String>| chips.set(list),
                    reset: search_reset(),
                }
                ViewToggle {
                    view: view.get(),
                    on_change: move |next: RecipeView| {
                        if next == RecipeView::Cards {
                            selected_rows.set(Vec::new());
                        }
                        view.set(next);
                    },
                }
                }
            }

            if view.get() == RecipeView::Table && !selected_rows().is_empty() {
                div { class: "mr-bulkbar",
                    Text { small: true, "{selected_rows().len()} selected" }
                    Button {
                        variant: ButtonVariant::Danger,
                        size: Size::Sm,
                        onclick: move |_| confirm_selected(),
                        "Delete selected"
                    }
                }
            }

            if !deleting().is_empty() {
                ConfirmDialog {
                    title: match deleting().as_slice() {
                        [only] => format!("Delete “{}”?", only.title),
                        many => format!("Delete {} recipes?", many.len()),
                    },
                    confirm_label: "Delete".to_string(),
                    danger: true,
                    loading: delete_busy(),
                    on_confirm: move |_| delete_confirmed(),
                    on_cancel: move |_| {
                        if !delete_busy() {
                            deleting.set(Vec::new());
                        }
                    },
                    Text {
                        "This removes it for good, along with its ingredient lines and tags. A recipe that is on a meal plan or in the cooking log can't be deleted."
                    }
                }
            }

            // ── The table, driven by the resource's three states.
            match &*recipes.read() {
                None => rsx! {
                    div { class: "mr-tablepanel mr-tablestate", Spinner {} }
                },
                Some(Err(e)) => rsx! {
                    Alert { tone: Tone::Red, title: "Couldn't load recipes", "{error_message(e)}" }
                },
                Some(Ok(_)) if no_matches => rsx! {
                    div { class: "mr-empty", role: "status", aria_live: "polite",
                        div { class: "mr-empty__icon", aria_hidden: "true", Icon { name: IconName::ChefHat } }
                        Heading { level: 3, "No recipes match {filter_summary}" }
                        Text { muted: true, "Try a different search, or clear the filters to see everything." }
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                query.set(String::new());
                                chips.set(Vec::new());
                                search_reset += 1;
                            },
                            "Clear filters"
                        }
                    }
                },
                Some(Ok(_)) if view.get() == RecipeView::Cards && !visible.read().is_empty() => rsx! {
                    div { class: "mr-recipegrid", aria_live: "polite",
                        for overview in visible() {
                            RecipeCard { key: "{overview.recipe.id}", overview }
                        }
                    }
                },
                Some(Ok(_)) => rsx! {
                    div { class: "mr-tablepanel", aria_live: "polite",
                        DataTable::<RecipeOverview> {
                            columns: columns.clone(),
                            rows: visible(),
                            empty: empty.to_string(),
                            selectable: true,
                            on_select: move |v: Vec<usize>| selected_rows.set(v),
                            on_row_click: open_row,
                        }
                    }
                },
            }
        }
    }
}
