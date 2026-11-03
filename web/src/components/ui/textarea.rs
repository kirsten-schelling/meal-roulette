use crate::components::ui::{field::error_id, field::use_control_id, Field};
use dioxus::prelude::*;

/// Multi-line text field — the `Input` counterpart for notes and instructions.
#[component]
pub fn Textarea(
    #[props(optional)] label: Option<String>,
    /// Controlled value — pass a signal's value and update it in `oninput`.
    value: String,
    oninput: EventHandler<FormEvent>,
    #[props(optional)] placeholder: Option<String>,
    /// Visible height in text rows. The user can still drag it taller.
    #[props(default = 4)]
    rows: u32,
    /// When set, shows the message in the danger color and flags the field.
    #[props(optional)]
    error: Option<String>,
) -> Element {
    let id = use_control_id();
    let has_error = error.is_some();

    rsx! {
        Field { label, error, control_id: id.clone(),
            textarea {
                id: "{id}",
                class: "mr-input mr-textarea",
                rows: "{rows}",
                value: "{value}",
                placeholder: placeholder.clone().unwrap_or_default(),
                aria_invalid: has_error.then_some("true"),
                aria_describedby: has_error.then(|| error_id(&id)),
                oninput: move |e| oninput.call(e),
            }
        }
    }
}
