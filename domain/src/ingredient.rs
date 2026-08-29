use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::unit::UnitEntry;

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct Ingredient {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset ingredient shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub purchase_size: i32,
    pub unit: String,
    pub unit_id: Uuid,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewIngredient {
    /// Optional for now — omit for a global ingredient. Becomes session-derived under auth.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub purchase_size: i32,
    pub unit_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIngredient {
    pub name: Option<String>,
    pub purchase_size: Option<i32>,
    pub unit_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum IngredientEntry {
    Existing(Uuid),
    New {
        name: String,
        purchase_size: i32,
        unit: UnitEntry,
    },
}
