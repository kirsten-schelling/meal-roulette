use crate::components::ui::{field::error_id, field::use_control_id, Field};
use dioxus::prelude::*;

/// Focuses the text control `id` names (or the first one inside it) and puts the caret
/// after its text, so a field opened by a tap is ready to type into. Inputs that have no
/// text caret (`number`) reject `setSelectionRange`, hence the `try`.
pub(crate) fn focus_at_end(id: &str) {
    let _ = document::eval(&FOCUS_AT_END.replace("__ID__", id));
}

const FOCUS_AT_END: &str = r#"
const root = document.getElementById("__ID__");
if (root) {
  const el = root.matches('input,textarea')
    ? root
    : root.querySelector('input:not([disabled]),textarea:not([disabled])')
      || root.querySelector('button:not([disabled]),select:not([disabled]),[tabindex]:not([tabindex="-1"])');
  if (el && el !== document.activeElement) {
    el.focus();
    try { const n = el.value.length; el.setSelectionRange(n, n); } catch (_) {}
  }
}
"#;

#[component]
pub fn Input(
    #[props(optional)] label: Option<String>,
    /// Controlled value — pass a signal's value and update it in `oninput`.
    value: String,
    oninput: EventHandler<FormEvent>,
    /// HTML input type: "text", "number", "email", … Defaults to "text".
    #[props(default = "text".to_string())]
    kind: String,
    #[props(optional)] placeholder: Option<String>,
    /// When set, shows the message in the danger color and flags the field.
    #[props(optional)]
    error: Option<String>,
    #[props(optional)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    /// Which on-screen keyboard a phone shows (`decimal`, `url`, …).
    #[props(optional)]
    inputmode: Option<String>,
    /// Label of the keyboard's Enter key (`next`, `done`, …).
    #[props(optional)]
    enterkeyhint: Option<String>,
    /// `off` stops the browser's own suggestions covering the form.
    #[props(optional)]
    autocomplete: Option<String>,
    #[props(optional)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    /// Takes focus when it appears, for a field the user just opened by clicking.
    #[props(default)]
    autofocus: bool,
) -> Element {
    let id = use_control_id();
    let has_error = error.is_some();

    rsx! {
        Field { label, error, required, control_id: id.clone(),
            input {
                id: "{id}",
                class: "mr-input",
                r#type: "{kind}",
                value: "{value}",
                placeholder: placeholder.clone().unwrap_or_default(),
                required,
                disabled,
                inputmode,
                enterkeyhint,
                autocomplete,
                aria_required: required.then_some("true"),
                aria_invalid: has_error.then_some("true"),
                aria_describedby: has_error.then(|| error_id(&id)),
                onmounted: {
                    let id = id.clone();
                    move |_| {
                        if autofocus {
                            focus_at_end(&id);
                        }
                    }
                },
                oninput: move |e| oninput.call(e),
                onkeydown: move |e| {
                    if let Some(k) = onkeydown {
                        k.call(e)
                    }
                },
                onblur: move |e| {
                    if let Some(b) = onblur {
                        b.call(e)
                    }
                },
            }
        }
    }
}
