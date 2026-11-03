use crate::models::parse_iso_date;
use dioxus::prelude::*;
use time::Date;

/// The browser's local calendar date, for marking "today". `None` until the
/// browser has answered, and on the server, where there is no browser to ask.
pub fn use_today() -> Memo<Option<Date>> {
    let found = use_resource(browser_today);
    use_memo(move || found().flatten())
}

async fn browser_today() -> Option<Date> {
    let value = document::eval(
        "const d = new Date(); \
         const pad = (n) => String(n).padStart(2, '0'); \
         return d.getFullYear() + '-' + pad(d.getMonth() + 1) + '-' + pad(d.getDate());",
    )
    .await
    .ok()?;
    parse_iso_date(value.as_str()?)
}
