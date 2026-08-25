use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `recipe` — also the JSON response shape.
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct Recipe {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset recipe shared by everyone.
    pub household_id: Option<Uuid>,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub servings: i16,
    pub healthiness: Option<i16>,
    pub main_protein_id: Option<Uuid>,
    pub cuisine_id: Option<Uuid>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /recipes`.
#[derive(Debug, Deserialize)]
pub struct NewRecipe {
    /// Optional for now — omit for a global recipe. Becomes session-derived under auth.
    pub household_id: Option<Uuid>,
    pub title: String,
    pub difficulty: i16,
    pub time_mins: i16,
    pub servings: i16,
    pub healthiness: Option<i16>,
    pub main_protein_id: Option<Uuid>,
    pub cuisine_id: Option<Uuid>,
}

/// Partial update body for `PUT /recipes/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateRecipe {
    pub title: Option<String>,
    pub difficulty: Option<i16>,
    pub time_mins: Option<i16>,
    pub servings: Option<i16>,
    pub healthiness: Option<i16>,
    pub main_protein_id: Option<Uuid>,
    pub cuisine_id: Option<Uuid>,
}
