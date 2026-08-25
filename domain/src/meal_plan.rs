use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `meal_plan` — one rolled week (the plan header).
#[derive(Debug, Serialize)]
pub struct MealPlan {
    pub id: Uuid,
    pub household_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub start_date: time::Date,
    pub seed: Option<i64>,
    pub name: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /meal-plans`. Dates are ISO `YYYY-MM-DD`.
#[derive(Debug, Deserialize)]
pub struct NewMealPlan {
    pub household_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub start_date: time::Date,
    pub seed: Option<i64>,
    pub name: Option<String>,
}

/// Partial update body for `PUT /meal-plans/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateMealPlan {
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub start_date: Option<time::Date>,
    pub seed: Option<i64>,
    pub name: Option<String>,
}
