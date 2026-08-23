use dioxus::prelude::*;

/// A padded surface container. Pass `header` for a titled bar, or omit it for
/// a plain padded box. Everything you nest goes in the body.
#[component]
pub fn Card(#[props(optional)] header: Option<String>, children: Element) -> Element {
    rsx! {
        div { class: "mr-card",
            if let Some(h) = header.clone() {
                div { class: "mr-card__header", "{h}" }
            }
            div { class: "mr-card__body", {children} }
        }
    }
}
