use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `cuisine` — lookup vocabulary for a recipe's cuisine.
#[derive(Debug, Serialize)]
pub struct Cuisine {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset value shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /cuisines`.
#[derive(Debug, Deserialize)]
pub struct NewCuisine {
    pub household_id: Option<Uuid>,
    pub name: String,
}

/// Partial update body for `PUT /cuisines/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateCuisine {
    pub name: Option<String>,
}
