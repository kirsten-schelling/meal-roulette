use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Longest sleep the shopping list offers, in weeks.
pub const MAX_SLEEP_WEEKS: i32 = 52;

/// A row from `slept_ingredient` — an ingredient kept off the list until a wake date.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SleptIngredient {
    pub id: Uuid,
    pub household_id: Uuid,
    pub ingredient_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub sleep_until: time::Date,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /slept-ingredients`. Dates are ISO `YYYY-MM-DD`.
#[derive(Debug, Deserialize)]
pub struct NewSleptIngredient {
    pub household_id: Uuid,
    pub ingredient_id: Uuid,
    #[serde(with = "crate::date_format::iso_date")]
    pub sleep_until: time::Date,
}

/// Partial update body for `PUT /slept-ingredients/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateSleptIngredient {
    pub ingredient_id: Option<Uuid>,
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub sleep_until: Option<time::Date>,
}
