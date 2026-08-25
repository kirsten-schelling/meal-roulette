//! ISO 8601 date (`YYYY-MM-DD`) serde for `time::Date` fields on JSON bodies.
//!
//! `time`'s default serde encodes a `Date` as a component array (e.g. `[2026, 236]`),
//! which is unusable from non-Rust clients. Fields carrying a calendar date use
//! `#[serde(with = "crate::date_format::iso_date")]` (or `::option` for `Option<Date>`)
//! to (de)serialize the standard `YYYY-MM-DD` string instead.

time::serde::format_description!(pub iso_date, Date, "[year]-[month]-[day]");
