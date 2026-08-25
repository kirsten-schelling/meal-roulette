use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `shopping_list` — a generated list (the header).
#[derive(Debug, Serialize)]
pub struct ShoppingList {
    pub id: Uuid,
    pub household_id: Uuid,
    /// The plan this list was generated from, if any. `None` = ad-hoc.
    pub meal_plan_id: Option<Uuid>,
    pub name: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /shopping-lists`.
#[derive(Debug, Deserialize)]
pub struct NewShoppingList {
    pub household_id: Uuid,
    pub meal_plan_id: Option<Uuid>,
    pub name: Option<String>,
}

/// Partial update body for `PUT /shopping-lists/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateShoppingList {
    pub meal_plan_id: Option<Uuid>,
    pub name: Option<String>,
}
