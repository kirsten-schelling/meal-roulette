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

/// Whether a rule's count is a floor, a ceiling or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Bound {
    #[default]
    AtLeast,
    AtMost,
    Exactly,
}

impl Bound {
    pub fn has_floor(self) -> bool {
        matches!(self, Bound::AtLeast | Bound::Exactly)
    }

    pub fn has_ceiling(self) -> bool {
        matches!(self, Bound::AtMost | Bound::Exactly)
    }
}

/// "At least / at most / exactly `times` of the week's dinners are this
/// cuisine, protein, ingredient or tag."
/// Rules stored before bounds and importance existed read as at-least and
/// required, which is what they meant then.
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

/// What one day of the week may take: recipes no easier than `min_difficulty`'s
/// band, no harder than `max_difficulty`'s band, and no longer to cook than
/// `max_time_mins`. A field left `None` is not limited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DayLimits {
    #[serde(default)]
    pub min_difficulty: Option<DifficultyBand>,
    #[serde(default)]
    pub max_difficulty: Option<DifficultyBand>,
    #[serde(default)]
    pub max_time_mins: Option<u32>,
}

impl DayLimits {
    /// The difficulty levels (1..=5) the day allows, lowest first.
    pub fn levels(self) -> (i16, i16) {
        let low = self.min_difficulty.map_or(1, |band| band.levels().0);
        let high = self.max_difficulty.map_or(5, |band| band.levels().1);
        (low, high)
    }

    /// Whether any of the three limits is set.
    pub fn is_limited(self) -> bool {
        self.min_difficulty.is_some()
            || self.max_difficulty.is_some()
            || self.max_time_mins.is_some()
    }

    /// Whether the limits can be met by any recipe at all.
    pub fn is_valid(self) -> bool {
        let (low, high) = self.levels();
        low <= high && self.max_time_mins != Some(0)
    }
}

/// `limits` for roll day `day`, counted from the first day (0), in place of the
/// default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayLimitsAt {
    pub day: u16,
    pub limits: DayLimits,
}

/// Difficulty and cook-time limits by day number: every day follows `default`
/// unless the day has an override. Days are numbers, never dates, so a roll
/// does not depend on which weekday it starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DayLimitPlan {
    #[serde(default)]
    pub default: DayLimits,
    #[serde(default)]
    pub overrides: Vec<DayLimitsAt>,
}

impl DayLimitPlan {
    pub fn limits_for(&self, day: usize) -> DayLimits {
        self.overrides
            .iter()
            .rev()
            .find(|o| usize::from(o.day) == day)
            .map_or(self.default, |o| o.limits)
    }

    /// Whether every limit in the plan can be met by some recipe.
    pub fn is_valid(&self) -> bool {
        self.default.is_valid() && self.overrides.iter().all(|o| o.limits.is_valid())
    }
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
    /// Tag rules count the dinners whose recipe carries the tag.
    #[serde(default)]
    pub tags: Vec<CountRule>,
    #[serde(default)]
    pub difficulties: Vec<DifficultyRule>,
    /// Limits on what each day of the week may take. `None` = every day takes
    /// any recipe, as plans rolled before day limits existed did.
    #[serde(default)]
    pub day_limits: Option<DayLimitPlan>,
}

/// Incoming body for `POST /roll`. Dates are ISO `YYYY-MM-DD`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollRequest {
    /// Owning household. `None` = the single local household.
    pub household_id: Option<Uuid>,
    /// Lay the dinners out on consecutive dates from here. `None` = leave them
    /// unscheduled, in roll order. Day limits are keyed by day number, so they
    /// apply either way.
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub start_date: Option<time::Date>,
    pub num_days: u16,
    /// Seed for the solver's random choices. `None` = a fresh random seed,
    /// which is returned on the plan so the roll can be repeated.
    pub seed: Option<i64>,
    #[serde(default)]
    pub constraints: RollConstraints,
    /// Dinners the roll keeps on their days. The rest of the week is rolled
    /// around them, and they count toward the rules.
    #[serde(default)]
    pub pinned: Vec<PinnedDinner>,
}

/// A recipe kept on roll day `day` (0 is the first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedDinner {
    pub day: u16,
    pub recipe_id: Uuid,
}

/// Which list in `RollConstraints` a rule sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleFamily {
    Cuisine,
    Protein,
    Ingredient,
    Tag,
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

/// One dinner of a roll that has not been kept: the recipe and what the
/// results list shows of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewMeal {
    pub recipe_id: Uuid,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub cuisine: Option<String>,
    pub protein: Option<String>,
    /// Cooked more recently than the recipe's minimum gap allows.
    pub within_min_gap: bool,
}

/// Response for `POST /roll/preview`: a roll that is not stored. `meals` are in
/// roll order and `seed` repeats the roll.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollPreview {
    pub seed: i64,
    pub meals: Vec<PreviewMeal>,
    pub unmet: Vec<UnmetRule>,
}

/// Incoming body for `POST /roll/preview/reroll`: replace the dinner on roll day
/// `day` (0 is the first) of an unsaved roll, keeping the others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RerollPreviewRequest {
    pub constraints: RollConstraints,
    /// The roll's recipes in roll order.
    pub recipe_ids: Vec<Uuid>,
    pub day: u16,
}

/// Response for `POST /roll/preview/reroll`: the new dinner, and the preferences
/// the roll falls short of with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RerollPreview {
    pub meal: PreviewMeal,
    pub unmet: Vec<UnmetRule>,
}

/// Incoming body for `POST /roll/keep`: store a previewed roll as a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeepRollRequest {
    pub household_id: Option<Uuid>,
    pub seed: Option<i64>,
    pub constraints: RollConstraints,
    /// The roll's recipes in roll order.
    pub recipe_ids: Vec<Uuid>,
}

/// Incoming body for `POST /roll/fit`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FitRequest {
    pub num_days: u16,
    pub constraints: RollConstraints,
}

/// How many recipes a roll can still draw on once its required rules and day
/// limits have ruled some out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FitCount {
    pub fitting: u32,
    pub total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_follows_the_default_unless_it_has_an_override() {
        let plan = DayLimitPlan {
            default: DayLimits {
                max_time_mins: Some(30),
                ..DayLimits::default()
            },
            overrides: vec![DayLimitsAt {
                day: 2,
                limits: DayLimits {
                    min_difficulty: Some(DifficultyBand::Hard),
                    ..DayLimits::default()
                },
            }],
        };
        assert_eq!(plan.limits_for(0).max_time_mins, Some(30));
        assert_eq!(plan.limits_for(2).levels(), (4, 5));
        assert_eq!(plan.limits_for(2).max_time_mins, None);
        assert_eq!(plan.limits_for(3).max_time_mins, Some(30));
    }

    #[test]
    fn plans_stored_without_day_limits_still_read() {
        let stored: RollConstraints = serde_json::from_str(r#"{"cuisines": []}"#).expect("parses");
        assert_eq!(stored.day_limits, None);
        let with: RollConstraints = serde_json::from_str(
            r#"{"day_limits": {"default": {"max_time_mins": 30}, "overrides": [{"day": 4, "limits": {}}]}}"#,
        )
        .expect("parses");
        let plan = with.day_limits.expect("present");
        assert_eq!(plan.default.max_time_mins, Some(30));
        assert!(!plan.limits_for(4).is_limited());
    }

    #[test]
    fn constraints_stored_before_tag_rules_and_exactly_still_read() {
        let stored: RollConstraints = serde_json::from_str(
            r#"{"cuisines": [{"id": "01a04351-39b7-7761-8bfb-58dad6017b5e", "times": 2, "bound": "AtMost"}]}"#,
        )
        .expect("parses");
        assert!(stored.tags.is_empty());
        assert_eq!(stored.cuisines[0].bound, Bound::AtMost);
        let exactly: CountRule = serde_json::from_str(
            r#"{"id": "01a04351-39b7-7761-8bfb-58dad6017b5e", "times": 1, "bound": "Exactly"}"#,
        )
        .expect("parses");
        assert!(exactly.bound.has_floor() && exactly.bound.has_ceiling());
    }

    #[test]
    fn limits_that_no_recipe_could_meet_are_invalid() {
        let backwards = DayLimits {
            min_difficulty: Some(DifficultyBand::Hard),
            max_difficulty: Some(DifficultyBand::Easy),
            max_time_mins: None,
        };
        assert!(!backwards.is_valid());
        assert!(!DayLimits {
            max_time_mins: Some(0),
            ..DayLimits::default()
        }
        .is_valid());
        assert!(DayLimits::default().is_valid());
    }
}
