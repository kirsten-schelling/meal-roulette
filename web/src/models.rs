use serde::{Deserialize, Serialize};

/// A recipe, as returned by the Rust API. Shared by the client and the
/// `#[server]` functions (crosses the wire, hence Serialize/Deserialize).
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub difficulty: i32,
    pub time_mins: i32,
}

pub fn difficulty_label(d: i32) -> &'static str {
    match d {
        1 => "Very easy",
        2 => "Easy",
        3 => "Medium",
        4 => "Hard",
        5 => "Ambitious",
        _ => "—",
    }
}
