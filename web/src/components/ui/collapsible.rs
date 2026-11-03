use crate::components::ui::field::use_control_id;
use dioxus::prelude::*;

/// A titled section whose body can be folded away. The header is one button;
/// `summary` is a line of text beside the title that stays visible when the body
/// is folded, so the section can be read without opening it. Controlled: pass
/// `open` and update it in `ontoggle`.
#[component]
pub fn Collapsible(
    title: String,
    #[props(optional)] summary: Option<String>,
    open: bool,
    ontoggle: EventHandler<bool>,
    children: Element,
) -> Element {
    let id = use_control_id();
    let chevron = if open {
        "mr-sel__chevron mr-sel__chevron--open"
    } else {
        "mr-sel__chevron"
    };

    rsx! {
        section { class: "mr-collapsible",
            button {
                r#type: "button",
                class: "mr-collapsible__head",
                aria_expanded: open,
                aria_controls: "{id}",
                onclick: move |_| ontoggle.call(!open),
                span { class: "mr-collapsible__title", "{title}" }
                if let Some(text) = summary.clone() {
                    span { class: "mr-collapsible__summary", "{text}" }
                }
                span { class: "{chevron}" }
            }
            if open {
                div { id: "{id}", class: "mr-collapsible__body", {children} }
            }
        }
    }
}
