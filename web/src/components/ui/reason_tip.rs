use super::popover::use_on_scroll;
use dioxus::prelude::*;

/// Wraps a control (usually a disabled button) and, while `reasons` is not empty, shows them
/// in a small bubble on hover or focus. A disabled button can't explain itself; this does.
/// Escape or scrolling hides the bubble until the pointer or focus comes back.
#[component]
pub fn ReasonTip(
    reasons: Vec<String>,
    #[props(default = "Still needed".to_string())] title: String,
    children: Element,
) -> Element {
    let id = use_hook(|| format!("mr-tip-{}", dioxus::core::current_scope_id().0));
    let mut dismissed = use_signal(|| false);
    use_on_scroll(move || dismissed.set(true));

    if reasons.is_empty() {
        return rsx! {
            {children}
        };
    }

    rsx! {
        span {
            class: if dismissed() { "mr-reasontip mr-reasontip--dismissed" } else { "mr-reasontip" },
            tabindex: "0",
            aria_describedby: "{id}",
            onmouseenter: move |_| dismissed.set(false),
            onfocusin: move |_| dismissed.set(false),
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    dismissed.set(true);
                }
            },
            {children}
            span { id: "{id}", class: "mr-reasontip__bubble", role: "tooltip",
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
