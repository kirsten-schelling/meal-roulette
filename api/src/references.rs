use domain::usage::{Reference, ReferenceKind};
use uuid::Uuid;

/// A referencing record as the reference queries select it: its id and display label.
pub struct ReferenceRow {
    pub id: Uuid,
    pub name: String,
}

pub fn to_references(rows: Vec<ReferenceRow>, kind: ReferenceKind) -> Vec<Reference> {
    rows.into_iter()
        .map(|row| Reference {
            kind,
            id: row.id,
            name: row.name,
        })
        .collect()
}
