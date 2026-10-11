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
