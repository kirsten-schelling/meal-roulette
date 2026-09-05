use crate::usage::UsageRef;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `protein` — lookup vocabulary for a recipe's main protein.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Protein {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset value shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// A protein plus the recipes that use it as their main protein.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProteinWithRecipeList {
    pub protein: Protein,
    pub used_by: Vec<UsageRef>,
}

/// Incoming body for `POST /proteins`.
#[derive(Debug, Serialize, Deserialize)]
pub struct NewProtein {
    pub household_id: Option<Uuid>,
    pub name: String,
}

/// Partial update body for `PUT /proteins/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateProtein {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum ProteinEntry {
    Existing(Uuid),
    New(String),
}
