use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `shopping_item`: something bought that is not a recipe ingredient.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShoppingItem {
    pub id: Uuid,
    pub name: String,
    /// The shopping list section it goes under. `None` = uncategorised.
    pub shopping_category_id: Option<Uuid>,
}
