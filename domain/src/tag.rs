use crate::usage::UsageRef;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `tag` — free-form recipe label.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tag {
    pub id: Uuid,
    /// Owning household. `None` = a global/preset value shared by everyone.
    pub household_id: Option<Uuid>,
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// A tag plus the recipes carrying it.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TagWithRecipeList {
    pub tag: Tag,
    pub used_by: Vec<UsageRef>,
}

/// A tag as a recipe lists it.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TagRef {
    pub id: Uuid,
    pub name: String,
}

/// Incoming body for `POST /tags`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NewTag {
    pub household_id: Option<Uuid>,
    pub name: String,
}

/// Partial update body for `PUT /tags/{id}`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdateTag {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum TagEntry {
    Existing(Uuid),
    New(String),
}
