use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `unit` — also the JSON response shape.
#[derive(Debug, Serialize)]
pub struct Unit {
    pub id: Uuid,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub name: String,
    pub description: Option<String>,
}

/// Incoming body for `POST /units`.
#[derive(Debug, Deserialize)]
pub struct NewUnit {
    pub name: String,
    pub description: Option<String>,
}

/// Partial update body for `PUT /units/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateUnit {
    pub name: Option<String>,
    pub description: Option<String>,
}
