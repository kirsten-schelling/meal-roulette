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

/// One ingredient line from the page, resolved both ways so the dialog can switch between them
/// without another round trip: against a library ingredient (`suggestion`) or as a new one.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct ImportedIngredient {
    /// The ingredient wording left after amount, unit, sizes and preparation are stripped.
    pub phrase: String,
    pub suggestion: Option<MatchSuggestion>,
    pub as_new: ImportedLine,
    /// True when the suggestion is confident enough to start as the chosen one.
    pub use_suggestion: bool,
}

/// A library ingredient the phrase looks like, and the line that results from choosing it.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct MatchSuggestion {
    pub confidence: f32,
    pub kind: MatchKind,
    pub line: ImportedLine,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum MatchKind {
    Exact,
    Remembered,
    Singular,
    Similar,
}

/// A recipe line as the dialog holds it. `ingredient` is `Existing` for a library match,
/// otherwise a `New` one for the user to confirm.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct ImportedLine {
    pub name: String,
    pub unit: String,
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    pub ingredient: IngredientEntry,
}
