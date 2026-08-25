use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `protein` — lookup vocabulary for a recipe's main protein.
#[derive(Debug, Serialize)]
pub struct Protein {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset value shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /proteins`.
#[derive(Debug, Deserialize)]
pub struct NewProtein {
    pub household_id: Option<Uuid>,
    pub name: String,
}

/// Partial update body for `PUT /proteins/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateProtein {
    pub name: Option<String>,
}
