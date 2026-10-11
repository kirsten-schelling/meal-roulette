use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ingredient::IngredientEntry;

/// A recipe-ingredient line. Reads are enriched with the ingredient's name and
/// unit (joined) so a recipe's ingredient list is usable on its own.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct RecipeIngredient {
    pub id: Uuid,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub unit: String,
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    pub is_optional: bool,
}

/// Incoming body for `POST /recipes/{id}/ingredients` (recipe_id comes from the path).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct NewRecipeIngredient {
    pub ingredient_id: Uuid,
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    #[serde(default)]
    pub is_optional: bool,
}

/// A brand-new ingredient created inline while building a recipe. Its unit is
/// either an existing `unit_id` or a `new_unit` name to be created first; the
/// qty/note/is_optional describe the recipe line once the ingredient exists.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct NewRecipeIngredientDraft {
    pub name: String,
    pub purchase_size: Option<i32>,
    pub unit_id: Option<Uuid>,
    pub new_unit: Option<String>,
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    #[serde(default)]
    pub is_optional: bool,
}

/// Body for `PUT /recipes/{id}/ingredients/{ingredient_id}`: the whole line, so the note can
/// also be cleared.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateRecipeIngredient {
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    pub is_optional: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct RecipeIngredientEntry {
    pub ingredient: IngredientEntry,
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    pub is_optional: bool,
}
