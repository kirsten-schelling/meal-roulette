use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `shopping_list_item` — one aggregated line on a list.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShoppingListItem {
    pub id: Uuid,
    pub shopping_list_id: Uuid,
    /// `None` for a line the household added that is not an ingredient (see `shopping_item_id`).
    pub ingredient_id: Option<Uuid>,
    /// Snapshot of the unit at generation time. `None` for a free-text line.
    pub unit_id: Option<Uuid>,
    pub qty: Option<Decimal>,
    pub is_purchased: bool,
    pub note: Option<String>,
    /// The shopping item of a line that is not an ingredient; `None` on an ingredient line.
    pub shopping_item_id: Option<Uuid>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// One list line joined with what the page shows beside it: the ingredient's
/// name and pack size, the snapshot unit's name, the recipes on the plan that
/// use it, and whether the household has it asleep.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShoppingListLine {
    /// The `shopping_list_item` id.
    pub id: Uuid,
    /// `None` for a free-text line, which has no ingredient to sleep.
    pub ingredient_id: Option<Uuid>,
    /// The ingredient's name, or the text of a free-text line.
    pub ingredient: String,
    pub qty: Option<Decimal>,
    /// Empty for a free-text line.
    pub unit: String,
    pub purchase_size: Option<i32>,
    /// Whole packs of `purchase_size` needed to cover `qty`; `None` unless both are known.
    pub packs: Option<i32>,
    pub is_purchased: bool,
    /// The pantry holds all of it: the line keeps its full amount but is not to buy.
    pub is_covered: bool,
    pub note: Option<String>,
    /// Titles of the plan's recipes that call for it, in date order.
    pub used_in: Vec<String>,
    /// The shopping category of the ingredient or shopping item behind the line.
    pub category_id: Option<Uuid>,
    /// Wake date of an active sleep. `None` = not asleep, so it goes on the list.
    #[serde(default, with = "crate::date_format::iso_date::option")]
    pub sleep_until: Option<time::Date>,
}

/// Incoming body for `POST /shopping-lists/{id}/custom-items`.
#[derive(Debug, Deserialize)]
pub struct NewCustomItem {
    pub name: String,
}

/// Incoming body for `POST /shopping-list-items`.
#[derive(Debug, Deserialize)]
pub struct NewShoppingListItem {
    pub shopping_list_id: Uuid,
    pub ingredient_id: Uuid,
    pub unit_id: Uuid,
    pub qty: Option<Decimal>,
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
