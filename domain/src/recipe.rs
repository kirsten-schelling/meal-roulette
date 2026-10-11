use crate::{
    cuisine::CuisineEntry,
    protein::ProteinEntry,
    recipe_ingredient::RecipeIngredientEntry,
    tag::{TagEntry, TagRef},
};

use super::recipe_ingredient::RecipeIngredient;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `recipe` — also the JSON response shape.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct Recipe {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset recipe shared by everyone.
    pub household_id: Option<Uuid>,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub servings: i16,
    pub healthiness: Option<i16>,
    pub main_protein_id: Option<Uuid>,
    pub cuisine_id: Option<Uuid>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct RecipeWithIngredients {
    #[serde(flatten)]
    pub recipe: Recipe,
    pub ingredients: Option<Vec<RecipeIngredient>>,
}

/// Incoming body for `POST /recipes`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewRecipe {
    /// Optional for now — omit for a global recipe. Becomes session-derived under auth.
    pub household_id: Option<Uuid>,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub servings: i16,
    pub healthiness: Option<i16>,
    pub protein: ProteinEntry,
    pub cuisine: CuisineEntry,
    pub ingredients: Vec<RecipeIngredientEntry>,
    #[serde(default)]
    pub tags: Vec<TagEntry>,
}

/// The resolved columns actually written to `recipe` — built server-side after
/// any inline cuisine/protein have been created and their ids filled in. Not a
/// wire type; it only carries the row.
#[derive(Debug)]
pub struct InsertRecipe {
    pub household_id: Option<Uuid>,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub servings: i16,
    pub healthiness: Option<i16>,
    pub main_protein_id: Option<Uuid>,
    pub cuisine_id: Option<Uuid>,
}

/// Body for `PUT /recipes/{id}`: the recipe's details as a whole, so a field can also be
/// cleared (`healthiness: None`). Ingredient lines are edited one at a time instead.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateRecipe {
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub servings: i16,
    pub healthiness: Option<i16>,
    pub protein: ProteinEntry,
    pub cuisine: CuisineEntry,
    pub tags: Vec<TagEntry>,
}

/// A recipe with its lookup values resolved to names, for lists and filters.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct RecipeOverview {
    pub recipe: Recipe,
    pub cuisine: Option<String>,
    pub protein: Option<String>,
    pub tags: Vec<TagRef>,
}

/// Everything the single-recipe page shows.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct RecipeDetail {
    pub overview: RecipeOverview,
    pub ingredients: Vec<RecipeIngredient>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct RecipeCount {
    pub cuisine_id: Uuid,
    pub count: i64,
}
