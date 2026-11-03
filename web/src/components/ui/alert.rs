use super::badge::Tone;
use dioxus::prelude::*;

/// A block-level status message. Shares the `Tone` color set with `Badge`.
/// Pass an optional bold `title` above the body content.
#[component]
pub fn Alert(
    #[props(default)] tone: Tone,
    #[props(optional)] title: Option<String>,
    children: Element,
) -> Element {
    let cls = match tone {
        Tone::Green => "mr-alert--green",
        Tone::Amber => "mr-alert--amber",
        Tone::Red => "mr-alert--red",
        Tone::Neutral => "mr-alert--neutral",
    };
    rsx! {
        div {
            class: "mr-alert {cls}",
            role: if tone == Tone::Red { "alert" } else { "status" },
            if let Some(t) = title.clone() {
                p { class: "mr-alert__title", "{t}" }
            }
            div { {children} }
        }
    }
}
