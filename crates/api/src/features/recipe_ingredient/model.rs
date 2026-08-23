use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A recipe-ingredient line. Reads are enriched with the ingredient's name and
/// unit (joined) so a recipe's ingredient list is usable on its own.
#[derive(Debug, Serialize)]
pub struct RecipeIngredient {
    pub id: Uuid,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub unit: String,
    pub qty: Decimal,
    pub note: Option<String>,
    pub optional: bool,
}

/// Incoming body for `POST /recipes/{id}/ingredients` (recipe_id comes from the path).
#[derive(Debug, Deserialize)]
pub struct NewRecipeIngredient {
    pub ingredient_id: Uuid,
    pub qty: Decimal,
    pub note: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

/// Partial update body for `PUT /recipes/{id}/ingredients/{ingredient_id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateRecipeIngredient {
    pub qty: Option<Decimal>,
    pub note: Option<String>,
    pub optional: Option<bool>,
}
