use dioxus::prelude::*;
use time::UtcOffset;

/// The browser's current UTC offset, for showing stored timestamps in local
/// time. UTC until the browser has answered, and on the server, where there is
/// no browser to ask.
pub fn use_local_offset() -> Memo<UtcOffset> {
    let found = use_resource(browser_offset);
    use_memo(move || found().flatten().unwrap_or(UtcOffset::UTC))
}

async fn browser_offset() -> Option<UtcOffset> {
    let value = document::eval("return new Date().getTimezoneOffset();")
        .await
        .ok()?;
    // getTimezoneOffset counts minutes behind UTC, so UTC+11 reads as -660.
    let minutes_behind = value.as_i64()?;
    let seconds = i32::try_from(-minutes_behind * 60).ok()?;
    UtcOffset::from_whole_seconds(seconds).ok()
}
