use crate::components::ui::{Icon, IconButton, IconName};
use dioxus::prelude::*;

/// Runs once the dialog is on screen. Moves focus into it (the first control in
/// the body, so a confirm dialog lands on Cancel, or the dialog itself when it is
/// marked `data-focus="dialog"`), keeps Tab and Shift+Tab inside
/// it, and hands focus back to whatever had it before once the dialog is gone.
pub(crate) const FOCUS_SCRIPT: &str = r#"
const id = "__ID__";
const dlg = document.getElementById(id);
if (dlg) {
  const prev = document.activeElement;
  const sel = 'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';
  const focusables = () => Array.from(dlg.querySelectorAll(sel)).filter((el) => el.getClientRects().length > 0);
  if (dlg.dataset.focus === 'dialog') {
    dlg.focus();
  } else if (!dlg.contains(document.activeElement)) {
    const bodyEl = dlg.querySelector('.mr-modal__body');
    const inBody = bodyEl ? Array.from(bodyEl.querySelectorAll(sel)) : [];
    const target = inBody.find((el) => el.getClientRects().length > 0) || focusables()[0] || dlg;
    target.focus();
  }
  dlg.addEventListener('keydown', (e) => {
    if (e.key !== 'Tab') return;
    const items = focusables();
    if (items.length === 0) { e.preventDefault(); dlg.focus(); return; }
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && (document.activeElement === first || document.activeElement === dlg)) {
      e.preventDefault(); last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault(); first.focus();
    }
  });
  const watcher = new MutationObserver(() => {
    if (!dlg.isConnected) {
      watcher.disconnect();
      if (prev && prev.isConnected && typeof prev.focus === 'function') prev.focus();
    }
  });
  watcher.observe(document.body, { childList: true, subtree: true });
}
"#;

/// A centered dialog over a dimmed backdrop. Pressing on the backdrop, pressing
/// Escape, or clicking the × calls `on_close`. Mount it conditionally —
/// `if show() { Modal { .. } }` — so every open starts with fresh state.
///
/// It is a modal dialog for assistive tech (`role="dialog"`, `aria-modal`, named
/// by its title). Focus moves into it when it opens, Tab stays inside, and focus
/// returns to the opener when it closes.
///
/// The backdrop closes on `mousedown`, not `click`, so a drag that *starts*
/// inside the dialog (e.g. selecting text) and releases on the backdrop doesn't
/// close it — only a press that begins on the backdrop does.
///
/// `focus_dialog` lands focus on the dialog rather than its first field, so a form
/// opens without a caret in it or a phone keyboard over it.
///
/// `wide` widens the dialog for form-heavy content (the default suits short
/// confirmations). The dialog is never taller than the viewport: the title stays
/// put and the body scrolls, with a `mr-dialog-foot` action row pinned to its bottom.
#[component]
pub fn Modal(
    on_close: EventHandler<()>,
    #[props(optional)] title: Option<String>,
    #[props(default)] wide: bool,
    #[props(default)] focus_dialog: bool,
    children: Element,
) -> Element {
    let id = use_hook(|| format!("mr-modal-{}", dioxus::core::current_scope_id().0));
    let title_id = format!("{id}-title");

    let script_id = id.clone();
    use_effect(move || {
        let _ = document::eval(&FOCUS_SCRIPT.replace("__ID__", &script_id));
    });

    let mut modal_cls = String::from("mr-modal");
    if wide {
        modal_cls.push_str(" mr-modal--wide");
    }

    rsx! {
        div {
            class: "mr-modal-overlay",
            onmousedown: move |_| on_close.call(()),
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    on_close.call(());
                }
            },
            div {
                id: "{id}",
                class: "{modal_cls}",
                role: "dialog",
                aria_modal: "true",
                aria_labelledby: title.is_some().then(|| title_id.clone()),
                tabindex: "-1",
                "data-focus": focus_dialog.then_some("dialog"),
                // A press that starts inside the dialog must not reach the
                // backdrop, or a drag-select ending outside would close it.
                onmousedown: move |e| e.stop_propagation(),
                if let Some(t) = title.clone() {
                    div { class: "mr-modal__header",
                        span { id: "{title_id}", "{t}" }
                        IconButton {
                            label: "Close".to_string(),
                            onclick: move |_| on_close.call(()),
                            Icon { name: IconName::X }
                        }
                    }
                }
                div { class: "mr-modal__body", {children} }
            }
        }
    }
}
