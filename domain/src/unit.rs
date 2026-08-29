use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `unit` — also the JSON response shape.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Unit {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset unit shared by everyone.
    pub household_id: Option<Uuid>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub name: String,
    pub description: Option<String>,
}

/// Incoming body for `POST /units`.
#[derive(Debug, Serialize, Deserialize)]
pub struct NewUnit {
    /// Optional for now — omit for a global unit. Becomes session-derived under auth.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
}

/// Partial update body for `PUT /units/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateUnit {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum UnitEntry {
    Existing(Uuid),
    New(String),
}
