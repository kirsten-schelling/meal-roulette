use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Ingredient {
    pub id: Uuid,
    pub name: String,
    pub purchase_size: i32,
    pub unit: String,
    pub unit_id: Uuid,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct NewIngredient {
    pub name: String,
    pub purchase_size: i32,
    pub unit_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIngredient {
    pub name: Option<String>,
    pub purchase_size: Option<i32>,
    pub unit_id: Option<Uuid>,
}
