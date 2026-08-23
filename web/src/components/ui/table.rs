use dioxus::prelude::*;

/// A styled, horizontally-scrollable table shell. You supply the standard
/// `thead`/`tbody`/`tr`/`th`/`td` markup as children — this just wraps it so
/// the `mr-table` styling applies and narrow screens scroll instead of squish.
#[component]
pub fn Table(children: Element) -> Element {
    rsx! {
        div { class: "mr-table-wrap",
            table { class: "mr-table", {children} }
        }
    }
}
