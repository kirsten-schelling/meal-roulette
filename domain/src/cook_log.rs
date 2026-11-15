use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `cook_log` — the permanent record of what was actually cooked.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CookLog {
    pub id: Uuid,
    pub household_id: Uuid,
    pub recipe_id: Uuid,
    /// The plan slot this cook came from, if any. `None` = unplanned.
    pub planned_meal_id: Option<Uuid>,
    #[serde(with = "crate::date_format::iso_date")]
    pub cooked_on: time::Date,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// A cook as the log lists it: the row plus the recipe's title.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CookLogEntry {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub title: String,
    #[serde(with = "crate::date_format::iso_date")]
    pub cooked_on: time::Date,
}

/// Incoming body for `POST /cook-logs`. Dates are ISO `YYYY-MM-DD`.
#[derive(Debug, Deserialize)]
pub struct NewCookLog {
    pub household_id: Uuid,
    pub recipe_id: Uuid,
    pub planned_meal_id: Option<Uuid>,
    #[serde(with = "crate::date_format::iso_date")]
    pub cooked_on: time::Date,
}

/// Partial update body for `PUT /cook-logs/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateCookLog {
    pub recipe_id: Option<Uuid>,
    pub planned_meal_id: Option<Uuid>,
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub cooked_on: Option<time::Date>,
}
