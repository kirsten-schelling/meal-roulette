use dioxus::prelude::ServerFnError;
use rust_decimal::Decimal;
use time::macros::format_description;
use time::Date;

/// The server's own message, without the "error running server function: … (details: None)"
/// wrapper `ServerFnError` adds when displayed.
pub fn error_message(error: &ServerFnError) -> String {
    match error {
        ServerFnError::ServerError { message, .. } => message.clone(),
        other => other.to_string(),
    }
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

/// "1.5 kg", trailing zeros dropped; "As needed" for a line that gives no amount.
pub fn amount_label(qty: Option<Decimal>, unit: &str) -> String {
    match qty {
        Some(qty) => format!("{} {unit}", qty.normalize()),
        None => "As needed".to_string(),
    }
}

/// "Mon 6 Oct": how a plan lists its days.
pub fn day_label(date: Date) -> String {
    date.format(format_description!(
        "[weekday repr:short] [day padding:none] [month repr:short]"
    ))
    .unwrap_or_else(|_| date.to_string())
}

/// How a dinner is named in a plan: "Mon 6 Oct" once it has a date, "Day 3" while it is
/// still in roll order.
pub fn dinner_day_label(day_index: i16, date: Option<Date>) -> String {
    match date {
        Some(date) => day_label(date),
        None => format!("Day {day_index}"),
    }
}

/// "6 Oct 2026".
pub fn date_label(date: Date) -> String {
    date.format(format_description!(
        "[day padding:none] [month repr:short] [year]"
    ))
    .unwrap_or_else(|_| date.to_string())
}

/// The `YYYY-MM-DD` value an HTML date input produces.
pub fn parse_iso_date(value: &str) -> Option<Date> {
    Date::parse(value.trim(), format_description!("[year]-[month]-[day]")).ok()
}

/// "Monday": the large label on a day card.
pub fn weekday_label(date: Date) -> String {
    date.format(format_description!("[weekday repr:long]"))
        .unwrap_or_else(|_| date.to_string())
}

/// "6 Oct": the date under a weekday on a day card.
pub fn short_date_label(date: Date) -> String {
    date.format(format_description!("[day padding:none] [month repr:short]"))
        .unwrap_or_else(|_| date.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn a_dinner_is_named_by_its_date_once_scheduled_and_by_its_roll_position_before() {
        assert_eq!(
            dinner_day_label(3, Some(date!(2026 - 11 - 06))),
            "Fri 6 Nov"
        );
        assert_eq!(dinner_day_label(3, None), "Day 3");
    }
}
