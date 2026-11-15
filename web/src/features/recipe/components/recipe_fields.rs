use crate::components::ui::{ComboOption, Combobox, Input, Picker, Rating, RatingScale};
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::library::components::ManageTab;
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::recipe::components::recipe_seed::RecipeSeed;
use crate::features::recipe::components::tag_picker::{TagChoice, TagPicker};
use crate::features::tag::hooks::use_tag::use_get_tags;
use dioxus::prelude::*;
use domain::cuisine::{CuisineEntry, CuisineWithRecipeList};
use domain::protein::{ProteinEntry, ProteinWithRecipeList};
use domain::recipe::{NewRecipe, RecipeOverview, UpdateRecipe};
use domain::recipe_ingredient::RecipeIngredientEntry;
use domain::tag::TagWithRecipeList;
use uuid::Uuid;

/// Flattens a lookup fetch into combobox rows: empty while loading or on error.
pub fn options_from<T>(
    fetched: &Option<Result<Vec<T>, ServerFnError>>,
    to_option: impl Fn(&T) -> ComboOption,
) -> Vec<ComboOption> {
    match fetched {
        Some(Ok(items)) => items.iter().map(to_option).collect(),
        _ => Vec::new(),
    }
}

/// A count typed into a number field: `Ok` once it is a whole number of at least 1.
pub fn positive_count(value: &str, name: &str) -> Result<i16, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{name} is required"));
    }
    match value.parse::<i16>() {
        Ok(n) if n >= 1 => Ok(n),
        Ok(_) => Err(format!("{name} must be at least 1")),
        Err(_) => Err(format!("{name} must be a whole number")),
    }
}

/// The catalogs a recipe's lookup fields pick from. The form's owner holds them so it can
/// refetch after the manage-lists dialog closes.
#[derive(Clone, Copy)]
pub struct RecipeCatalogs {
    cuisines: Resource<Result<Vec<CuisineWithRecipeList>, ServerFnError>>,
    proteins: Resource<Result<Vec<ProteinWithRecipeList>, ServerFnError>>,
    tags: Resource<Result<Vec<TagWithRecipeList>, ServerFnError>>,
}

pub fn use_recipe_catalogs() -> RecipeCatalogs {
    RecipeCatalogs {
        cuisines: use_get_cuisines(),
        proteins: use_get_proteins(),
        tags: use_get_tags(),
    }
}

impl RecipeCatalogs {
    pub fn restart(mut self) {
        self.cuisines.restart();
        self.proteins.restart();
        self.tags.restart();
    }

    pub fn cuisine_options(&self) -> Vec<ComboOption> {
        options_from(&(self.cuisines)(), |c| {
            ComboOption::new(c.cuisine.id, c.cuisine.name.clone())
        })
    }

    pub fn protein_options(&self) -> Vec<ComboOption> {
        options_from(&(self.proteins)(), |p| {
            ComboOption::new(p.protein.id, p.protein.name.clone())
        })
    }

    pub fn tag_options(&self) -> Vec<ComboOption> {
        options_from(&(self.tags)(), |t| {
            ComboOption::new(t.tag.id, t.tag.name.clone())
        })
    }
}

/// What the form holds, compared against how it started to tell whether anything changed.
#[derive(Clone, PartialEq)]
struct Snapshot {
    title: String,
    difficulty: i16,
    time_mins: String,
    servings: String,
    healthiness: Option<i16>,
    cuisine: Picker,
    protein: Picker,
    tags: Vec<TagChoice>,
}

fn picked(id: Option<Uuid>, name: Option<&String>) -> Picker {
    let mut picker = Picker::default();
    if let (Some(id), Some(name)) = (id, name) {
        picker.pick(ComboOption::new(id, name.clone()));
    }
    picker
}

impl Snapshot {
    fn starting(initial: Option<&RecipeOverview>) -> Self {
        match initial {
            None => Snapshot {
                title: String::new(),
                difficulty: 3,
                time_mins: String::new(),
                servings: "2".to_string(),
                healthiness: None,
                cuisine: Picker::default(),
                protein: Picker::default(),
                tags: Vec::new(),
            },
            Some(o) => Snapshot {
                title: o.recipe.title.clone(),
                difficulty: o.recipe.difficulty,
                time_mins: o.recipe.time_mins.to_string(),
                servings: o.recipe.servings.to_string(),
                healthiness: o.recipe.healthiness,
                cuisine: picked(o.recipe.cuisine_id, o.cuisine.as_ref()),
                protein: picked(o.recipe.main_protein_id, o.protein.as_ref()),
                tags: o.tags.iter().map(TagChoice::from).collect(),
            },
        }
    }
}

/// A recipe's details as editable state, shared by the add and edit dialogs.
#[derive(Clone, Copy, PartialEq)]
pub struct RecipeForm {
    pub title: Signal<String>,
    pub difficulty: Signal<i16>,
    pub time_mins: Signal<String>,
    pub servings: Signal<String>,
    pub healthiness: Signal<Option<i16>>,
    pub cuisine: Signal<Picker>,
    pub protein: Signal<Picker>,
    pub tags: Signal<Vec<TagChoice>>,
    time_touched: Signal<bool>,
    servings_touched: Signal<bool>,
    baseline: Signal<Snapshot>,
}

/// Starts empty for a new recipe, or filled from `initial` for an edit.
pub fn use_recipe_form(initial: Option<RecipeOverview>) -> RecipeForm {
    let start = use_hook(|| Snapshot::starting(initial.as_ref()));
    RecipeForm {
        title: use_signal(|| start.title.clone()),
        difficulty: use_signal(|| start.difficulty),
        time_mins: use_signal(|| start.time_mins.clone()),
        servings: use_signal(|| start.servings.clone()),
        healthiness: use_signal(|| start.healthiness),
        cuisine: use_signal(|| start.cuisine.clone()),
        protein: use_signal(|| start.protein.clone()),
        tags: use_signal(|| start.tags.clone()),
        time_touched: use_signal(|| false),
        servings_touched: use_signal(|| false),
        baseline: use_signal(|| start.clone()),
    }
}

impl RecipeForm {
    /// Refills every field from `overview`, so an edit starts from what is saved now.
    pub fn load(mut self, overview: &RecipeOverview) {
        let start = Snapshot::starting(Some(overview));
        self.title.set(start.title.clone());
        self.difficulty.set(start.difficulty);
        self.time_mins.set(start.time_mins.clone());
        self.servings.set(start.servings.clone());
        self.healthiness.set(start.healthiness);
        self.cuisine.set(start.cuisine.clone());
        self.protein.set(start.protein.clone());
        self.tags.set(start.tags.clone());
        self.time_touched.set(false);
        self.servings_touched.set(false);
        self.baseline.set(start);
    }

    /// Fills the fields `seed` names and counts them as the starting point, so
    /// closing the form untouched does not ask to discard them.
    pub fn prefill(mut self, seed: &RecipeSeed) {
        if let Some(option) = &seed.cuisine {
            self.cuisine.write().pick(option.clone());
        }
        if let Some(option) = &seed.protein {
            self.protein.write().pick(option.clone());
        }
        if let Some(tag) = &seed.tag {
            self.tags.write().push(tag.clone());
        }
        if let Some(difficulty) = seed.difficulty {
            self.difficulty.set(difficulty);
        }
        self.baseline.set(self.snapshot());
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            title: (self.title)(),
            difficulty: (self.difficulty)(),
            time_mins: (self.time_mins)(),
            servings: (self.servings)(),
            healthiness: (self.healthiness)(),
            cuisine: (self.cuisine)(),
            protein: (self.protein)(),
            tags: (self.tags)(),
        }
    }

    /// Anything changed since the form opened.
    pub fn is_dirty(&self) -> bool {
        *self.baseline.read() != self.snapshot()
    }

    /// The required fields still empty or invalid, in the order they appear.
    pub fn missing(&self) -> Vec<String> {
        let mut missing = Vec::new();
        if (self.title)().trim().is_empty() {
            missing.push("A title".to_string());
        }
        if !(self.protein)().is_resolved() {
            missing.push("A main protein".to_string());
        }
        let cuisine = (self.cuisine)();
        if !cuisine.is_resolved() && !cuisine.query.trim().is_empty() {
            missing.push("A cuisine picked from the list, or Enter to create it".to_string());
        }
        if let Err(e) = positive_count(&(self.time_mins)(), "Time") {
            missing.push(e);
        }
        if let Err(e) = positive_count(&(self.servings)(), "Servings") {
            missing.push(e);
        }
        missing
    }

    fn parts(&self) -> Option<Parts> {
        if !self.missing().is_empty() {
            return None;
        }
        Some(Parts {
            title: (self.title)().trim().to_string(),
            difficulty: (self.difficulty)(),
            time_mins: positive_count(&(self.time_mins)(), "Time").ok()?,
            servings: positive_count(&(self.servings)(), "Servings").ok()?,
            healthiness: (self.healthiness)(),
            cuisine: {
                let cuisine = (self.cuisine)();
                cuisine
                    .is_resolved()
                    .then(|| cuisine.resolve(CuisineEntry::Existing, CuisineEntry::New))
            },
            protein: (self.protein)().resolve(ProteinEntry::Existing, ProteinEntry::New),
            tags: (self.tags)().iter().map(TagChoice::entry).collect(),
        })
    }

    pub fn new_recipe(&self, ingredients: Vec<RecipeIngredientEntry>) -> Option<NewRecipe> {
        let p = self.parts()?;
        Some(NewRecipe {
            household_id: None,
            title: p.title,
            difficulty: p.difficulty,
            time_mins: p.time_mins,
            servings: p.servings,
            healthiness: p.healthiness,
            protein: p.protein,
            cuisine: p.cuisine,
            ingredients,
            tags: p.tags,
        })
    }

    pub fn update(&self) -> Option<UpdateRecipe> {
        let p = self.parts()?;
        Some(UpdateRecipe {
            title: p.title,
            difficulty: p.difficulty,
            time_mins: p.time_mins,
            servings: p.servings,
            healthiness: p.healthiness,
            protein: p.protein,
            cuisine: p.cuisine,
            tags: p.tags,
        })
    }
}

struct Parts {
    title: String,
    difficulty: i16,
    time_mins: i16,
    servings: i16,
    healthiness: Option<i16>,
    cuisine: Option<CuisineEntry>,
    protein: ProteinEntry,
    tags: Vec<domain::tag::TagEntry>,
}

/// The recipe's own details (everything but its ingredient lines). `children` render
/// right under the title.
#[component]
pub fn RecipeFields(
    form: RecipeForm,
    cuisine_options: Vec<ComboOption>,
    protein_options: Vec<ComboOption>,
    tag_options: Vec<ComboOption>,
    on_manage: EventHandler<ManageTab>,
    children: Element,
) -> Element {
    let RecipeForm {
        mut title,
        mut difficulty,
        mut time_mins,
        mut servings,
        mut healthiness,
        cuisine,
        protein,
        tags,
        mut time_touched,
        mut servings_touched,
        ..
    } = form;

    let time_error = time_touched()
        .then(|| positive_count(&time_mins(), "Time").err())
        .flatten();
    let servings_error = servings_touched()
        .then(|| positive_count(&servings(), "Servings").err())
        .flatten();

    rsx! {
        Input {
            label: "Title".to_string(),
            value: title(),
            required: true,
            oninput: move |e: FormEvent| title.set(e.value()),
            placeholder: "e.g. Pad Thai".to_string(),
        }
        {children}
        div { class: "mr-form-grid",
            Combobox {
                label: "Main protein".to_string(),
                placeholder: "Search, or type a new one and press Enter…".to_string(),
                options: protein_options.clone(),
                picker: protein,
                required: true,
                manage_label: "Manage proteins…".to_string(),
                on_manage: move |_| on_manage.call(ManageTab::Proteins),
            }
            Combobox {
                label: "Cuisine".to_string(),
                placeholder: "Search, or type a new one and press Enter…".to_string(),
                options: cuisine_options.clone(),
                picker: cuisine,
                manage_label: "Manage cuisines…".to_string(),
                on_manage: move |_| on_manage.call(ManageTab::Cuisines),
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
                required: true,
                oninput: move |e: FormEvent| {
                    time_mins.set(e.value());
                    time_touched.set(true);
                },
                onblur: move |_| time_touched.set(true),
                placeholder: "e.g. 25".to_string(),
                error: time_error,
            }
            Input {
                label: "Servings".to_string(),
                kind: "number".to_string(),
                value: servings(),
                required: true,
                oninput: move |e: FormEvent| {
                    servings.set(e.value());
                    servings_touched.set(true);
                },
                onblur: move |_| servings_touched.set(true),
                error: servings_error,
            }
        }
        TagPicker {
            label: "Tags".to_string(),
            options: tag_options.clone(),
            selected: tags,
            on_manage: move |_| on_manage.call(ManageTab::Tags),
        }
    }
}
