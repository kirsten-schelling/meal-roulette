use time::macros::format_description;
use time::Date;

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
