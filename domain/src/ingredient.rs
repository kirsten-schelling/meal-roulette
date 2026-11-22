use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::unit::UnitEntry;

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct Ingredient {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset ingredient shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub purchase_size: Option<i32>,
    /// The unit's name; an empty string means the ingredient has no unit.
    pub unit: String,
    pub unit_id: Option<Uuid>,
    /// The shopping list section it goes under. `None` = uncategorised.
    pub shopping_category_id: Option<Uuid>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /ingredients`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewIngredient {
    /// Optional for now — omit for a global ingredient. Becomes session-derived under auth.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub purchase_size: Option<i32>,
    pub unit: Option<UnitEntry>,
    #[serde(default)]
    pub shopping_category_id: Option<Uuid>,
}

/// The resolved columns actually written to `ingredient` — built server-side after
/// an inline unit has been created and its id filled in. Not a wire type; it only
/// carries the row.
#[derive(Debug)]
pub struct InsertIngredient {
    pub household_id: Option<Uuid>,
    pub name: String,
    pub purchase_size: Option<i32>,
    pub unit_id: Option<Uuid>,
    pub shopping_category_id: Option<Uuid>,
}

/// Body for `PUT /ingredients/{id}`. Like `NewIngredient`, the unit may be one to create.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateIngredient {
    pub name: String,
    pub purchase_size: Option<i32>,
    pub unit: Option<UnitEntry>,
    #[serde(default)]
    pub shopping_category_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum IngredientEntry {
    Existing(Uuid),
    New {
        name: String,
        purchase_size: Option<i32>,
        unit: Option<UnitEntry>,
    },
}
