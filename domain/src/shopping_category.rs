use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `shopping_category`: a section the shopping list is grouped by.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShoppingCategory {
    pub id: Uuid,
    pub name: String,
    /// Where the section sits on the list; lower comes first.
    pub position: i32,
}
