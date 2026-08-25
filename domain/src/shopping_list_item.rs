use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `shopping_list_item` — one aggregated line on a list.
#[derive(Debug, Serialize)]
pub struct ShoppingListItem {
    pub id: Uuid,
    pub shopping_list_id: Uuid,
    pub ingredient_id: Uuid,
    /// Snapshot of the unit at generation time.
    pub unit_id: Uuid,
    pub qty: Decimal,
    pub is_purchased: bool,
    pub note: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /shopping-list-items`.
#[derive(Debug, Deserialize)]
pub struct NewShoppingListItem {
    pub shopping_list_id: Uuid,
    pub ingredient_id: Uuid,
    pub unit_id: Uuid,
    pub qty: Decimal,
    #[serde(default)]
    pub is_purchased: bool,
    pub note: Option<String>,
}

/// Partial update body for `PUT /shopping-list-items/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateShoppingListItem {
    pub ingredient_id: Option<Uuid>,
    pub unit_id: Option<Uuid>,
    pub qty: Option<Decimal>,
    pub is_purchased: Option<bool>,
    pub note: Option<String>,
}
