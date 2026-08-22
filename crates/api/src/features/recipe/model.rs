use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `recipes` — also the JSON response shape.
#[derive(Debug, Serialize)]
pub struct Recipe {
    pub id: Uuid,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /recipes`.
#[derive(Debug, Deserialize)]
pub struct NewRecipe {
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
}

// Partial update body for `PUT /recipes/:id`.
#[derive(Debug, Deserialize)]
pub struct UpdateRecipe {
    pub title: Option<String>,
    pub difficulty: Option<i16>,
    pub time_mins: Option<i16>,
}
