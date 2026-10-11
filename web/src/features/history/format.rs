//! Labels for a rolled plan, shared by the history, the plan page and the
//! shopping list.

use crate::models::day_label;
use time::macros::format_description;
use time::{Date, OffsetDateTime, UtcOffset};

/// "Mon 5 Oct to Sun 11 Oct", or just the first day when the plan covers one
/// day or has none.
pub fn week_label(start: Date, end: Option<Date>) -> String {
    match end {
        Some(end) if end != start => format!("{} to {}", day_label(start), day_label(end)),
        _ => day_label(start),
    }
}

/// "11 Oct 2026, 4:52 pm" at `offset`.
pub fn rolled_label(at: OffsetDateTime, offset: UtcOffset) -> String {
    let local = at.checked_to_offset(offset).unwrap_or(at);
    local
        .format(format_description!(
            "[day padding:none] [month repr:short] [year], [hour repr:12 padding:none]:[minute] [period case:lower]"
        ))
        .unwrap_or_else(|_| local.to_string())
}

/// "1 dinner", "5 dinners".
pub fn dinner_count(count: i64) -> String {
    if count == 1 {
        "1 dinner".to_string()
    } else {
        format!("{count} dinners")
    }
}
