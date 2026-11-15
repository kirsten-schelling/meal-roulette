use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One ingredient the household has an estimated count of, as the pantry page shows it.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PantryLine {
    pub ingredient_id: Uuid,
    pub ingredient: String,
    /// Empty when the ingredient has no unit.
    pub unit: String,
    pub qty: Decimal,
}
