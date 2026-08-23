use dioxus::prelude::*;

/// A centered dialog over a dimmed backdrop. Controlled via `open`; clicking the
/// backdrop or the × calls `on_close`. Renders nothing when closed.
#[component]
pub fn Modal(
    open: bool,
    on_close: EventHandler<()>,
    #[props(optional)] title: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        if open {
            div {
                class: "mr-modal-overlay",
                onclick: move |_| on_close.call(()),
                div {
                    class: "mr-modal",
                    // don't let clicks inside the dialog reach the backdrop
                    onclick: move |e| e.stop_propagation(),
                    if let Some(t) = title.clone() {
                        div { class: "mr-modal__header",
                            span { "{t}" }
                            button {
                                class: "mr-modal__close",
                                aria_label: "Close",
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
}
