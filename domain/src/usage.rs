use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One record that references a lookup value: a recipe using a cuisine or
/// protein, an ingredient measured in a unit. `name` is its display label.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UsageRef {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    Recipe,
    Ingredient,
    ShoppingList,
}

/// A record standing in the way of deleting a lookup value, with enough to link to it.
/// A shopping list stands for the item(s) on it that use the value.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Reference {
    pub kind: ReferenceKind,
    pub id: Uuid,
    pub name: String,
}
