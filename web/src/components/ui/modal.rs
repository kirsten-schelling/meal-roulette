use crate::components::ui::IconButton;
use dioxus::prelude::*;

/// A centered dialog over a dimmed backdrop. Pressing on the backdrop or
/// clicking the × calls `on_close`. Mount it conditionally —
/// `if show() { Modal { .. } }` — so every open starts with fresh state.
///
/// The backdrop closes on `mousedown`, not `click`, so a drag that *starts*
/// inside the dialog (e.g. selecting text) and releases on the backdrop doesn't
/// close it — only a press that begins on the backdrop does.
///
/// `wide` widens the dialog for form-heavy content (the default suits short
/// confirmations). `overflow_visible` lets descendant popups (dropdowns,
/// comboboxes) escape the dialog box instead of being clipped by it.
#[component]
pub fn Modal(
    on_close: EventHandler<()>,
    #[props(optional)] title: Option<String>,
    #[props(default)] wide: bool,
    #[props(default)] overflow_visible: bool,
    children: Element,
) -> Element {
    let mut modal_cls = String::from("mr-modal");
    if wide {
        modal_cls.push_str(" mr-modal--wide");
    }
    if overflow_visible {
        modal_cls.push_str(" mr-modal--overflow");
    }

    rsx! {
        div {
            class: "mr-modal-overlay",
            onmousedown: move |_| on_close.call(()),
            div {
                class: "{modal_cls}",
                // A press that starts inside the dialog must not reach the
                // backdrop, or a drag-select ending outside would close it.
                onmousedown: move |e| e.stop_propagation(),
                if let Some(t) = title.clone() {
                    div { class: "mr-modal__header",
                        span { "{t}" }
                        IconButton {
                            label: "Close".to_string(),
                            onclick: move |_| on_close.call(()),
                            "\u{00d7}"
                        }
                    }
                }
                div { class: "mr-modal__body", {children} }
            }
        }
    }
}
