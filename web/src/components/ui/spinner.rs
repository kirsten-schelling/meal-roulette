use super::button::Size;
use dioxus::prelude::*;

/// A spinning loading indicator in the accent color. Size defaults to medium.
#[component]
pub fn Spinner(#[props(default)] size: Size) -> Element {
    let cls = match size {
        Size::Sm => "mr-spinner--sm",
        Size::Md => "mr-spinner--md",
        Size::Lg => "mr-spinner--lg",
    };
    rsx! {
        span {
            class: "mr-spinner mr-spinner--solo {cls}",
            role: "status",
            aria_label: "Loading",
        }
    }
}
