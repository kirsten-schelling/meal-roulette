use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `ingredient_alias`: a normalised recipe-page phrase and the library ingredient
/// the user chose for it.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IngredientAlias {
    pub id: Uuid,
    /// Owning household. `None` = shared by everyone.
    pub household_id: Option<Uuid>,
    pub ingredient_id: Uuid,
    pub phrase: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
