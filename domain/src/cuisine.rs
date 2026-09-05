use crate::usage::UsageRef;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `cuisine` — lookup vocabulary for a recipe's cuisine.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Cuisine {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset value shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// A cuisine plus the recipes that use it.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CuisineWithRecipeList {
    pub cuisine: Cuisine,
    pub used_by: Vec<UsageRef>,
}

/// Incoming body for `POST /cuisines`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NewCuisine {
    pub household_id: Option<Uuid>,
    pub name: String,
}

/// Partial update body for `PUT /cuisines/{id}`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdateCuisine {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum CuisineEntry {
    Existing(Uuid),
    New(String),
}
