use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One record that references a lookup value: a recipe using a cuisine or
/// protein, an ingredient measured in a unit. `name` is its display label.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UsageRef {
    pub id: Uuid,
    pub name: String,
}
