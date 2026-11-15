//! A saved recipe's details, each one edited in place: click a value to swap it for its
//! control, then save just that change.

use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{
    Badge, Combobox, Editable, Heading, InlineEditor, Input, Rating, RatingScale,
};
use crate::features::library::components::ManageTab;
use crate::features::recipe::cadence::describe;
use crate::features::recipe::components::cadence_picker::CadencePicker;
use crate::features::recipe::components::recipe_fields::{
    positive_count, use_recipe_form, RecipeCatalogs, RecipeForm,
};
use crate::features::recipe::components::tag_picker::TagPicker;
use crate::features::recipe::hooks::use_recipe::update_recipe;
use crate::features::recipe::page::healthiness_badge;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::recipe::{RecipeDetail, RecipeOverview};

#[derive(Clone, Copy, PartialEq)]
enum DetailField {
    Title,
    Difficulty,
    Healthiness,
    Time,
    Servings,
    Cuisine,
    Protein,
    Tags,
    Cadence,
}

const SUMMARY_FIELDS: [DetailField; 8] = [
    DetailField::Difficulty,
    DetailField::Healthiness,
    DetailField::Time,
    DetailField::Servings,
    DetailField::Cuisine,
    DetailField::Protein,
    DetailField::Tags,
    DetailField::Cadence,
];

impl DetailField {
    fn label(self) -> &'static str {
        match self {
            DetailField::Title => "Title",
            DetailField::Difficulty => "Difficulty",
            DetailField::Healthiness => "Healthiness",
            DetailField::Time => "Time",
            DetailField::Servings => "Servings",
            DetailField::Cuisine => "Cuisine",
            DetailField::Protein => "Main protein",
            DetailField::Tags => "Tags",
            DetailField::Cadence => "How often",
        }
    }
}

fn named(value: &Option<String>) -> Element {
    match value {
        Some(v) => rsx! { "{v}" },
        None => rsx! {
            span { class: "mr-text--muted", "None" }
        },
    }
}

/// How a detail reads when it is not being edited.
fn value(field: DetailField, overview: &RecipeOverview) -> Element {
    let recipe = &overview.recipe;
    match field {
        DetailField::Title => rsx! { "{recipe.title}" },
        DetailField::Difficulty => rsx! {
            DifficultyBadge { difficulty: recipe.difficulty }
        },
        DetailField::Healthiness => healthiness_badge(recipe.healthiness),
        DetailField::Time => rsx! {
            span { class: "mr-num", "{recipe.time_mins} min" }
        },
        DetailField::Servings => rsx! {
            span { class: "mr-num", "{recipe.servings}" }
        },
        DetailField::Cuisine => named(&overview.cuisine),
        DetailField::Protein => named(&overview.protein),
        DetailField::Tags if overview.tags.is_empty() => rsx! {
            span { class: "mr-text--muted", "No tags" }
        },
        DetailField::Tags => rsx! {
            span { class: "mr-tagcell",
                for tag in overview.tags.iter() {
                    Badge { key: "{tag.id}", "{tag.name}" }
                }
            }
        },
        DetailField::Cadence => rsx! { "{describe(recipe.min_gap_weeks, recipe.max_gap_weeks)}" },
    }
}

/// Which detail is open (one at a time) and the form it is edited through. Saving sends
/// the whole form, refilled from the saved recipe when the field opened, so only the
/// open field differs from what is stored.
#[derive(Clone, Copy)]
pub struct DetailsEditor {
    form: RecipeForm,
    detail: Resource<Result<RecipeDetail, ServerFnError>>,
    catalogs: RecipeCatalogs,
    managing: Signal<Option<ManageTab>>,
    open: Signal<Option<DetailField>>,
    saving: Signal<bool>,
    error: Signal<Option<String>>,
}

/// `detail` is refetched after each save so the page shows what was stored. The page
/// owns `catalogs` and `managing` so it can refetch the lookups once the manage-lists
/// dialog that `managing` opens is closed.
pub fn use_details_editor(
    detail: Resource<Result<RecipeDetail, ServerFnError>>,
    catalogs: RecipeCatalogs,
    managing: Signal<Option<ManageTab>>,
) -> DetailsEditor {
    DetailsEditor {
        form: use_recipe_form(None),
        detail,
        catalogs,
        managing,
        open: use_signal(|| None),
        saving: use_signal(|| false),
        error: use_signal(|| None),
    }
}

impl DetailsEditor {
    fn is_open(&self, field: DetailField) -> bool {
        *self.open.read() == Some(field)
    }

    fn start(mut self, field: DetailField, overview: &RecipeOverview) {
        self.form.load(overview);
        self.error.set(None);
        self.open.set(Some(field));
    }

    fn cancel(mut self) {
        self.open.set(None);
        self.error.set(None);
    }

    fn save(mut self, overview: &RecipeOverview) {
        if (self.saving)() {
            return;
        }
        if !self.form.is_dirty() {
            self.cancel();
            return;
        }
        let Some(update) = self.form.update() else {
            let missing = self.form.missing().join(", ");
            self.error.set(Some(format!("Still needed: {missing}")));
            return;
        };
        let id = overview.recipe.id;
        self.saving.set(true);
        self.error.set(None);
        spawn(async move {
            match update_recipe(id, update).await {
                Ok(_) => {
                    self.open.set(None);
                    self.detail.restart();
                }
                Err(e) => self.error.set(Some(error_message(&e))),
            }
            self.saving.set(false);
        });
    }

    fn editable(self, field: DetailField, overview: &RecipeOverview) -> Element {
        let o = overview.clone();
        rsx! {
            Editable {
                label: format!("Edit {}", field.label().to_lowercase()),
                onclick: move |_| self.start(field, &o),
                {value(field, overview)}
            }
        }
    }

    /// Wraps `control` with Save (unless it saves itself) and Cancel.
    fn editor(
        self,
        overview: &RecipeOverview,
        saves_itself: bool,
        enter_saves: bool,
        control: Element,
    ) -> Element {
        let on_save = (!saves_itself).then(|| {
            let o = overview.clone();
            EventHandler::new(move |_| self.save(&o))
        });
        rsx! {
            InlineEditor {
                on_save,
                on_cancel: move |_| self.cancel(),
                saving: (self.saving)(),
                error: (self.error)(),
                enter_saves,
                {control}
            }
        }
    }

    fn control(self, field: DetailField, overview: &RecipeOverview) -> Element {
        let (catalogs, mut managing) = (self.catalogs, self.managing);
        let RecipeForm {
            mut title,
            mut difficulty,
            mut healthiness,
            mut time_mins,
            mut servings,
            min_gap_weeks,
            max_gap_weeks,
            cuisine,
            protein,
            tags,
            ..
        } = self.form;
        match field {
            DetailField::Title => self.editor(
                overview,
                false,
                true,
                rsx! {
                    div { class: "mr-title-editor",
                        Input {
                            value: title(),
                            autofocus: true,
                            oninput: move |e: FormEvent| title.set(e.value()),
                            placeholder: "e.g. Pad Thai".to_string(),
                        }
                    }
                },
            ),
            DetailField::Difficulty => {
                let o = overview.clone();
                self.editor(
                    overview,
                    true,
                    false,
                    rsx! {
                        Rating {
                            value: Some(difficulty()),
                            scale: RatingScale::HighIsBad,
                            onchange: move |n: i16| {
                                difficulty.set(n);
                                self.save(&o);
                            },
                        }
                    },
                )
            }
            DetailField::Healthiness => {
                let (o, o_clear) = (overview.clone(), overview.clone());
                self.editor(
                    overview,
                    true,
                    false,
                    rsx! {
                        Rating {
                            value: healthiness(),
                            scale: RatingScale::HighIsGood,
                            onchange: move |n: i16| {
                                healthiness.set(Some(n));
                                self.save(&o);
                            },
                            on_clear: move |_| {
                                healthiness.set(None);
                                self.save(&o_clear);
                            },
                        }
                    },
                )
            }
            DetailField::Time => self.editor(
                overview,
                false,
                true,
                rsx! {
                    Input {
                        kind: "number".to_string(),
                        value: time_mins(),
                        autofocus: true,
                        oninput: move |e: FormEvent| time_mins.set(e.value()),
                        placeholder: "Minutes".to_string(),
                        error: positive_count(&time_mins(), "Time").err(),
                    }
                },
            ),
            DetailField::Servings => self.editor(
                overview,
                false,
                true,
                rsx! {
                    Input {
                        kind: "number".to_string(),
                        value: servings(),
                        autofocus: true,
                        oninput: move |e: FormEvent| servings.set(e.value()),
                        error: positive_count(&servings(), "Servings").err(),
                    }
                },
            ),
            DetailField::Cuisine => self.editor(
                overview,
                false,
                true,
                rsx! {
                    Combobox {
                        placeholder: "Search, or type a new one…".to_string(),
                        options: catalogs.cuisine_options(),
                        picker: cuisine,
                        manage_label: "Manage cuisines…".to_string(),
                        on_manage: move |_| managing.set(Some(ManageTab::Cuisines)),
                    }
                },
            ),
            DetailField::Protein => self.editor(
                overview,
                false,
                true,
                rsx! {
                    Combobox {
                        placeholder: "Search, or type a new one…".to_string(),
                        options: catalogs.protein_options(),
                        picker: protein,
                        manage_label: "Manage proteins…".to_string(),
                        on_manage: move |_| managing.set(Some(ManageTab::Proteins)),
                    }
                },
            ),
            DetailField::Tags => self.editor(
                overview,
                false,
                false,
                rsx! {
                    TagPicker {
                        options: catalogs.tag_options(),
                        selected: tags,
                        on_manage: move |_| managing.set(Some(ManageTab::Tags)),
                    }
                },
            ),
            DetailField::Cadence => self.editor(
                overview,
                false,
                false,
                rsx! {
                    CadencePicker {
                        label: None,
                        min_weeks: min_gap_weeks,
                        max_weeks: max_gap_weeks,
                    }
                },
            ),
        }
    }

    /// The page title, which turns into a text field when clicked.
    pub fn title(self, overview: &RecipeOverview) -> Element {
        if self.is_open(DetailField::Title) {
            return self.control(DetailField::Title, overview);
        }
        rsx! {
            Heading { level: 1, {self.editable(DetailField::Title, overview)} }
        }
    }

    /// The summary list, where every value opens its own control when clicked.
    pub fn summary(self, overview: &RecipeOverview) -> Element {
        rsx! {
            dl { class: "mr-summary",
                for field in SUMMARY_FIELDS {
                    div {
                        key: "{field.label()}",
                        class: if matches!(field, DetailField::Tags | DetailField::Cadence) { "mr-summary__item mr-summary__item--wide" } else { "mr-summary__item" },
                        dt { class: "mr-summary__label", "{field.label()}" }
                        dd { class: "mr-summary__value",
                            if self.is_open(field) {
                                {self.control(field, overview)}
                            } else {
                                {self.editable(field, overview)}
                            }
                        }
                    }
                }
            }
        }
    }
}
