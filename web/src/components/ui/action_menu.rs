use dioxus::prelude::*;
use std::rc::Rc;

/// A small labelled "More" button that opens a short list of secondary
/// actions. It is a disclosure, not an ARIA menu: the list sits right after
/// the button in tab order, Escape closes it and returns focus to the button,
/// and pressing anywhere else closes it. `label` names the button for
/// assistive tech and should start with "More", e.g. "More actions for Rice".
#[component]
pub fn ActionMenu(label: String, children: Element) -> Element {
    let mut open = use_signal(|| false);
    use_context_provider(|| open);
    let mut trigger = use_signal(|| None as Option<Rc<MountedData>>);

    rsx! {
        span {
            class: "mr-menu",
            onkeydown: move |e: KeyboardEvent| {
                if open() && e.key() == Key::Escape {
                    open.set(false);
                    if let Some(el) = trigger() {
                        spawn(async move {
                            let _ = el.set_focus(true).await;
                        });
                    }
                }
            },
            button {
                r#type: "button",
                class: "mr-btn mr-btn--ghost mr-btn--sm mr-menu__trigger",
                aria_label: "{label}",
                aria_expanded: open(),
                onmounted: move |e: MountedEvent| trigger.set(Some(e.data())),
                onclick: move |_| open.set(!open()),
                "More"
            }
            if open() {
                div {
                    class: "mr-menu__scrim",
                    aria_hidden: "true",
                    onclick: move |_| open.set(false),
                }
                div { class: "mr-menu__panel", {children} }
            }
        }
    }
}

/// One action inside an `ActionMenu`. Clicking it closes the menu, then runs
/// `onclick`. `danger` colours destructive actions.
#[component]
pub fn ActionMenuItem(
    #[props(default)] danger: bool,
    #[props(default)] disabled: bool,
    onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let mut open = use_context::<Signal<bool>>();
    let class = if danger {
        "mr-menu__item mr-menu__item--danger"
    } else {
        "mr-menu__item"
    };

    rsx! {
        button {
            r#type: "button",
            class: "{class}",
            disabled,
            onclick: move |e| {
                open.set(false);
                onclick.call(e);
            },
            {children}
        }
    }
}
