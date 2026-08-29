use crate::{
    cuisine::CuisineEntry, protein::ProteinEntry, recipe_ingredient::RecipeIngredientEntry,
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

/// Partial update body for `PUT /recipes/{id}`.
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateRecipe {
    pub title: Option<String>,
    pub difficulty: Option<i16>,
    pub time_mins: Option<i16>,
    pub servings: Option<i16>,
    pub healthiness: Option<i16>,
    pub main_protein_id: Option<Uuid>,
    pub cuisine_id: Option<Uuid>,
}
