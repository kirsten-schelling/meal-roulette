use crate::meal_plan::MealPlanSummary;
use crate::shopping_list_item::ShoppingListLine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `shopping_list` — a generated list (the header).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShoppingList {
    pub id: Uuid,
    pub household_id: Uuid,
    /// The plan this list was generated from, if any. `None` = ad-hoc.
    pub meal_plan_id: Option<Uuid>,
    pub name: Option<String>,
    pub created_at: time::OffsetDateTime,
    /// Bumped on every regeneration, so it doubles as "generated at".
    pub updated_at: time::OffsetDateTime,
}

/// A list with its lines, ready to display.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShoppingListDetail {
    pub list: ShoppingList,
    pub items: Vec<ShoppingListLine>,
    /// A day of the plan changed after the list was generated, so the lines
    /// may no longer match the plan.
    pub is_stale: bool,
}

/// A plan and its shopping list, if one has been generated.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShoppingListForPlan {
    pub plan: MealPlanSummary,
    pub list: Option<ShoppingListDetail>,
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
