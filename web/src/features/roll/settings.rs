use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The whole roll form, serialised into a preset's JSON payload. Field values
/// mirror the page's signals so a saved preset round-trips exactly.
#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RollSettings {
    #[serde(default)]
    pub num_days: String,
    /// Simple overall lean: "any" | "easy" | "balanced" | "ambitious".
    #[serde(default)]
    pub difficulty: String,
    /// When true, `difficulty_mix` overrides the simple lean.
    #[serde(default)]
    pub difficulty_exact: bool,
    /// Wanted days per difficulty level. Unassigned days roll at any level.
    #[serde(default)]
    pub difficulty_mix: Vec<DifficultyDays>,
    #[serde(default)]
    pub proteins: Vec<LookupRule>,
    #[serde(default)]
    pub cuisines: Vec<LookupRule>,
    #[serde(default)]
    pub ingredients: Vec<SavedIngredient>,
}

/// How many days should land on a difficulty level (1 = Very easy .. 5 = Ambitious).
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DifficultyDays {
    pub level: i16,
    pub days: u32,
}

/// A required protein or cuisine, with how many times it should appear.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LookupRule {
    pub id: Uuid,
    pub name: String,
    pub times: u32,
}

/// A must-use ingredient, kept with its label so chips render without a lookup.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedIngredient {
    pub id: Uuid,
    pub label: String,
}
