use crate::components::ui::{Icon, IconButton, IconName, Tone};
use dioxus::prelude::*;

/// The visual toast — a floating notification card with a tone icon and a
/// tone-colored edge. `title` is the headline; when present, the children
/// render underneath as a muted subtitle, otherwise they are the whole message.
/// Deliberately dumb: no timers, no queue, no global state. Render toasts
/// inside a `div.mr-toast-stack` (fixed, bottom-right) and own the show/
/// dismiss lifecycle in the app. Passing `on_dismiss` adds a × button.
#[component]
pub fn Toast(
    #[props(default)] tone: Tone,
    #[props(optional)] title: Option<String>,
    on_dismiss: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let tone_cls = match tone {
        Tone::Green => "mr-toast--green",
        Tone::Amber => "mr-toast--amber",
        Tone::Red => "mr-toast--red",
        Tone::Neutral => "mr-toast--neutral",
    };
    // Errors interrupt the screen reader. Everything else is announced politely by
    // the stack around it, a live region that is mounted before any toast appears.
    let role = matches!(tone, Tone::Red).then_some("alert");

    rsx! {
        div { class: "mr-toast {tone_cls}", role: role,
            span { class: "mr-toast__icon", aria_hidden: "true" }
            div { class: "mr-toast__body",
                if let Some(t) = title.clone() {
                    p { class: "mr-toast__title", "{t}" }
                }
                div { class: "mr-toast__msg", {children} }
            }
            if let Some(cb) = on_dismiss {
                IconButton {
                    label: "Dismiss".to_string(),
                    onclick: move |_| cb.call(()),
                    Icon { name: IconName::X }
                }
            }
        }
    }
}
