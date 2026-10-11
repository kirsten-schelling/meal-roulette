use crate::meal_plan::MealPlanDetail;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How much a rule matters. `Required` must hold or the roll fails; the others
/// are kept, most important first, as far as the recipes allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Importance {
    Low,
    Medium,
    High,
    #[default]
    Required,
}

/// Whether a rule's count is a floor or a ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Bound {
    #[default]
    AtLeast,
    AtMost,
}

/// "At least / at most `times` of the week's dinners are this cuisine, protein
/// or ingredient." Rules stored before bounds and importance existed read as
/// at-least and required, which is what they meant then.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountRule {
    pub id: Uuid,
    pub times: u32,
    #[serde(default)]
    pub bound: Bound,
    #[serde(default)]
    pub importance: Importance,
}

/// A group of adjacent difficulty levels (1 Very easy .. 5 Ambitious).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyBand {
    Easy,
    Medium,
    Hard,
}

impl DifficultyBand {
    pub const ALL: [DifficultyBand; 3] = [
        DifficultyBand::Easy,
        DifficultyBand::Medium,
        DifficultyBand::Hard,
    ];

    /// The difficulty levels the band covers, lowest first.
    pub fn levels(self) -> (i16, i16) {
        match self {
            DifficultyBand::Easy => (1, 2),
            DifficultyBand::Medium => (3, 3),
            DifficultyBand::Hard => (4, 5),
        }
    }
}

/// "At least / at most `times` of the week's dinners fall in this band."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifficultyRule {
    pub band: DifficultyBand,
    pub times: u32,
    #[serde(default)]
    pub bound: Bound,
    #[serde(default)]
    pub importance: Importance,
}

/// Every constraint family the solver understands. Stored on the plan it
/// produced, so a single day can be re-rolled against the same rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RollConstraints {
    #[serde(default)]
    pub cuisines: Vec<CountRule>,
    #[serde(default)]
    pub proteins: Vec<CountRule>,
    /// Ingredient rules count the dinners whose recipe uses the ingredient.
    #[serde(default)]
    pub ingredients: Vec<CountRule>,
    #[serde(default)]
    pub difficulties: Vec<DifficultyRule>,
}

/// Incoming body for `POST /roll`. Dates are ISO `YYYY-MM-DD`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollRequest {
    /// Owning household. `None` = the single local household.
    pub household_id: Option<Uuid>,
    /// First day of the plan. `None` = today.
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub start_date: Option<time::Date>,
    pub num_days: u16,
    /// Seed for the solver's random choices. `None` = a fresh random seed,
    /// which is returned on the plan so the roll can be repeated.
    pub seed: Option<i64>,
    #[serde(default)]
    pub constraints: RollConstraints,
}

/// Which list in `RollConstraints` a rule sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleFamily {
    Cuisine,
    Protein,
    Ingredient,
    Difficulty,
}

/// A preference the roll could not fully meet: the rule at `index` in its
/// family's list asked for `times` (`bound`), and the week has `got`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnmetRule {
    pub family: RuleFamily,
    pub index: usize,
    pub bound: Bound,
    pub times: u32,
    pub got: u32,
}

/// Response for `POST /roll`: the stored plan, and the preferences it had to loosen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollOutcome {
    pub plan: MealPlanDetail,
    pub unmet: Vec<UnmetRule>,
}
