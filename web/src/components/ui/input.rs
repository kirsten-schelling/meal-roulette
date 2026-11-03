use crate::components::ui::{field::error_id, field::use_control_id, Field};
use dioxus::prelude::*;

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
                aria_required: required.then_some("true"),
                aria_invalid: has_error.then_some("true"),
                aria_describedby: has_error.then(|| error_id(&id)),
                onmounted: move |e: MountedEvent| async move {
                    if autofocus {
                        let _ = e.set_focus(true).await;
                    }
                },
                oninput: move |e| oninput.call(e),
                onblur: move |e| {
                    if let Some(b) = onblur {
                        b.call(e)
                    }
                },
            }
        }
    }
}
