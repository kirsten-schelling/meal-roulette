use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// "This cuisine or protein must appear on `times` days of the week."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountRule {
    pub id: Uuid,
    pub times: u32,
}

/// Every constraint family the solver understands. Stored on the plan it
/// produced, so a single day can be re-rolled against the same rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RollConstraints {
    #[serde(default)]
    pub cuisines: Vec<CountRule>,
    #[serde(default)]
    pub proteins: Vec<CountRule>,
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
