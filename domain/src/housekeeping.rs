use serde::{Deserialize, Serialize};

/// Which library list a finding is about.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum LookupKind {
    Cuisine,
    Protein,
    Unit,
    Tag,
    Ingredient,
}

/// One thing the library needs attention for. New checks are new variants.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Finding {
    /// Values no recipe or ingredient uses.
    Unused {
        kind: LookupKind,
        names: Vec<String>,
    },
    /// Pairs of names that look like the same thing.
    NearDuplicates {
        kind: LookupKind,
        pairs: Vec<(String, String)>,
    },
    /// Ingredients with no purchase size set.
    MissingPurchaseSize { names: Vec<String> },
}
