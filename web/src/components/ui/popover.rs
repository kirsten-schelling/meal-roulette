use dioxus::prelude::*;
use std::rc::Rc;

const SCROLL_SCRIPT: &str = "(async () => { for (;;) { const e = await new Promise((r) => window.addEventListener('scroll', r, { capture: true, once: true, passive: true })); if (!(e.target instanceof Element && e.target.closest('.mr-popover__pop'))) { dioxus.send(true); return; } } })();";

/// Runs `on_scroll` whenever the page (or any scroller outside a popover card)
/// scrolls, so floating cards don't hang in place over moved content.
pub fn use_on_scroll(mut on_scroll: impl FnMut() + 'static) {
    use_hook(move || {
        spawn(async move {
            loop {
                let mut ev = document::eval(SCROLL_SCRIPT);
                if ev.recv::<bool>().await.is_err() {
                    break;
                }
                on_scroll();
            }
        })
    });
}

/// A small floating card that shows while its trigger is hovered or focused.
/// The trigger renders inside a real button, so keyboard users reach the card
/// too: focus opens it, Escape closes it. `label` names that button for
/// assistive tech, since the visible trigger is often just a number.
///
/// The card is `position: fixed`, placed from the trigger's measured rect, so
/// it escapes ancestors that scroll or clip (a row list inside a dialog). It
/// stays a DOM child of the trigger's wrapper, so moving the pointer from the
/// trigger into the card never counts as leaving.
#[component]
pub fn Popover(label: String, trigger: Element, children: Element) -> Element {
    let mut open = use_signal(|| false);
    let mut trigger_el = use_signal(|| None as Option<Rc<MountedData>>);
    // Bottom-right corner of the trigger in viewport pixels, once measured.
    let mut anchor = use_signal(|| None as Option<(f64, f64)>);

    let mut show = move || {
        open.set(true);
        let Some(el) = trigger_el() else { return };
        spawn(async move {
            if let Ok(rect) = el.get_client_rect().await {
                // The pointer may already have left while we measured.
                if open() {
                    anchor.set(Some((rect.max_x(), rect.max_y())));
                }
            }
        });
    };
    let mut hide = move || {
        open.set(false);
        anchor.set(None);
    };
    use_on_scroll(hide);

    rsx! {
        span {
            class: "mr-popover",
            onmouseenter: move |_| show(),
            onmouseleave: move |_| hide(),
            button {
                r#type: "button",
                class: "mr-popover__trigger",
                aria_label: "{label}",
                aria_expanded: open(),
                onmounted: move |e: MountedEvent| trigger_el.set(Some(e.data())),
                onfocus: move |_| show(),
                onblur: move |_| hide(),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Escape {
                        hide();
                    }
                },
                {trigger}
            }
            if let Some((x, y)) = anchor() {
                div { class: "mr-popover__pop", style: "top:{y}px;left:{x}px;",
                    div { class: "mr-popover__panel", {children} }
                }
            }
        }
    }
}
