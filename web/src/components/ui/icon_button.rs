use dioxus::prelude::*;

/// A small square button holding a single glyph (×, ＋, ✎). `label` is the
/// accessible name, since the glyph alone says nothing to a screen reader.
/// `danger` colors the hover state for destructive actions.
#[component]
pub fn IconButton(
    label: String,
    onclick: EventHandler<MouseEvent>,
    #[props(default)] danger: bool,
    children: Element,
) -> Element {
    let cls = if danger {
        "mr-iconbtn mr-iconbtn--danger"
    } else {
        "mr-iconbtn"
    };

    rsx! {
        button {
            r#type: "button",
            class: "{cls}",
            aria_label: "{label}",
            onclick: move |e| onclick.call(e),
            {children}
        }
    }
}
