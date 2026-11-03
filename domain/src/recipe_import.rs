use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ingredient::IngredientEntry;

/// A recipe read from a web page, for the add-recipe dialog to review. Nothing here is saved.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct RecipeImport {
    pub title: String,
    pub time_mins: Option<i16>,
    pub servings: Option<i16>,
    pub source_url: String,
    pub ingredients: Vec<ImportedIngredient>,
}

/// One ingredient line from the page. `ingredient` is `Existing` when the name matched a saved
/// ingredient, otherwise a `New` one for the user to confirm.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct ImportedIngredient {
    pub name: String,
    pub unit: String,
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    pub ingredient: IngredientEntry,
}
