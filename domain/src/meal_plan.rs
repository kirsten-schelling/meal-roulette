use crate::planned_meal::PlannedMealDetail;
use crate::roll::RollConstraints;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `meal_plan` — one rolled week (the plan header).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MealPlan {
    pub id: Uuid,
    pub household_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub start_date: time::Date,
    pub seed: Option<i64>,
    pub name: Option<String>,
    /// The rules this plan was rolled under.
    pub constraints: RollConstraints,
    pub is_favourite: bool,
    /// When the plan was archived. `None` = active.
    pub archived_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// One plan as the history lists it: when it was rolled, the week it covers,
/// and its dinner titles in date order.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MealPlanSummary {
    pub id: Uuid,
    pub household_id: Uuid,
    /// When the plan was rolled.
    pub created_at: time::OffsetDateTime,
    #[serde(with = "crate::date_format::iso_date")]
    pub start_date: time::Date,
    /// The last planned day. `None` = the plan has no days.
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub end_date: Option<time::Date>,
    pub num_days: i64,
    pub seed: Option<i64>,
    pub is_favourite: bool,
    pub archived_at: Option<time::OffsetDateTime>,
    pub dinners: Vec<String>,
}

/// A plan together with its dinners, one per day in date order.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MealPlanDetail {
    pub plan: MealPlan,
    pub meals: Vec<PlannedMealDetail>,
}

/// Incoming body for `POST /meal-plans`. Dates are ISO `YYYY-MM-DD`.
#[derive(Debug, Deserialize)]
pub struct NewMealPlan {
    pub household_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub start_date: time::Date,
    pub seed: Option<i64>,
    pub name: Option<String>,
    #[serde(default)]
    pub constraints: RollConstraints,
}

/// Partial update body for `PUT /meal-plans/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateMealPlan {
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub start_date: Option<time::Date>,
    pub seed: Option<i64>,
    pub name: Option<String>,
}
