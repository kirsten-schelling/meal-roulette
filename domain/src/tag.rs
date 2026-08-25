use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `tag` — free-form recipe label.
#[derive(Debug, Serialize)]
pub struct Tag {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset value shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /tags`.
#[derive(Debug, Deserialize)]
pub struct NewTag {
    pub household_id: Option<Uuid>,
    pub name: String,
}

/// Partial update body for `PUT /tags/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
}
