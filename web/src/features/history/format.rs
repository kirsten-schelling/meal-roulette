//! Labels for a rolled plan, shared by the history, the plan page and the
//! shopping list.

use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

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

/// What a plan is called: its name, or when it was rolled.
pub fn plan_title(name: Option<&str>, rolled_at: OffsetDateTime, offset: UtcOffset) -> String {
    match name.map(str::trim).filter(|name| !name.is_empty()) {
        Some(name) => name.to_string(),
        None => format!("Plan rolled {}", rolled_label(rolled_at, offset)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plan_is_named_or_else_dated_by_when_it_was_rolled() {
        let rolled = OffsetDateTime::UNIX_EPOCH;
        assert_eq!(
            plan_title(Some("Winter week"), rolled, UtcOffset::UTC),
            "Winter week"
        );
        assert_eq!(
            plan_title(Some("  "), rolled, UtcOffset::UTC),
            "Plan rolled 1 Jan 1970, 12:00 am"
        );
        assert_eq!(
            plan_title(None, rolled, UtcOffset::UTC),
            "Plan rolled 1 Jan 1970, 12:00 am"
        );
    }
}
