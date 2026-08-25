use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Household {
    pub id: Uuid,
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct NewHousehold {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHousehold {
    pub name: Option<String>,
}
