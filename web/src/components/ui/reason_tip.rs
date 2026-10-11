use dioxus::prelude::*;

/// Wraps a control (usually a disabled button) and, while `reasons` is not empty, shows them
/// in a small bubble on hover or focus. A disabled button can't explain itself; this does.
#[component]
pub fn ReasonTip(
    reasons: Vec<String>,
    #[props(default = "Still needed".to_string())] title: String,
    children: Element,
) -> Element {
    if reasons.is_empty() {
        return rsx! {
            {children}
        };
    }

    rsx! {
        span { class: "mr-reasontip", tabindex: "0",
            {children}
            span { class: "mr-reasontip__bubble", role: "tooltip",
                span { class: "mr-reasontip__title", "{title}" }
                ul {
                    for reason in reasons.iter() {
                        li { key: "{reason}", "{reason}" }
                    }
                }
            }
        }
    }
}
