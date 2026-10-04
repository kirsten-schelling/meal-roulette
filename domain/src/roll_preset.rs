use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A row from `roll_preset`: a named, saved set of roll-page settings. The
/// `settings` payload is opaque JSON owned by the web layer.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RollPreset {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset value shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub settings: Value,
    pub is_default: bool,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /roll-presets`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NewRollPreset {
    pub household_id: Option<Uuid>,
    pub name: String,
    pub settings: Value,
}

/// Partial update body for `PUT /roll-presets/{id}`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdateRollPreset {
    pub name: Option<String>,
    pub settings: Option<Value>,
}
