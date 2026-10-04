use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `planned_meal` — one dinner slot in a plan.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PlannedMeal {
    pub id: Uuid,
    pub meal_plan_id: Uuid,
    pub recipe_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub scheduled_date: time::Date,
    pub is_pinned: bool,
    /// Per-day difficulty override. `None` = use the day's default.
    pub target_difficulty: Option<i16>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// A dinner slot joined with the recipe filling it, ready to display: the
/// recipe's roll attributes plus the names behind its cuisine and protein ids.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PlannedMealDetail {
    pub id: Uuid,
    pub meal_plan_id: Uuid,
    pub recipe_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub scheduled_date: time::Date,
    pub is_pinned: bool,
    pub target_difficulty: Option<i16>,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub servings: i16,
    pub healthiness: Option<i16>,
    pub cuisine_id: Option<Uuid>,
    pub cuisine: Option<String>,
    pub main_protein_id: Option<Uuid>,
    pub protein: Option<String>,
}

/// Incoming body for `POST /planned-meals`. Dates are ISO `YYYY-MM-DD`.
#[derive(Debug, Deserialize)]
pub struct NewPlannedMeal {
    pub meal_plan_id: Uuid,
    pub recipe_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub scheduled_date: time::Date,
    #[serde(default)]
    pub is_pinned: bool,
    pub target_difficulty: Option<i16>,
}

/// Partial update body for `PUT /planned-meals/{id}`.
#[derive(Debug, Deserialize, Default)]
pub struct UpdatePlannedMeal {
    pub recipe_id: Option<Uuid>,
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub scheduled_date: Option<time::Date>,
    pub is_pinned: Option<bool>,
    pub target_difficulty: Option<i16>,
}
