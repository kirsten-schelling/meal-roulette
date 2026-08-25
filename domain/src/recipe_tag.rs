use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row from `recipe_tag` — attaches a tag to a recipe (many-to-many).
#[derive(Debug, Serialize)]
pub struct RecipeTag {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// Incoming body for `POST /recipe-tags`.
#[derive(Debug, Deserialize)]
pub struct NewRecipeTag {
    pub recipe_id: Uuid,
    pub tag_id: Uuid,
}
