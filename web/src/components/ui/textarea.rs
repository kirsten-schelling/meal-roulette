use crate::components::ui::Field;
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
    rsx! {
        Field { label, error,
            textarea {
                class: "mr-input mr-textarea",
                rows: "{rows}",
                value: "{value}",
                placeholder: placeholder.clone().unwrap_or_default(),
                oninput: move |e| oninput.call(e),
            }
        }
    }
}
