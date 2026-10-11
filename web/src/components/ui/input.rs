use crate::components::ui::Field;
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
) -> Element {
    rsx! {
        Field { label, error, required,
            input {
                class: "mr-input",
                r#type: "{kind}",
                value: "{value}",
                placeholder: placeholder.clone().unwrap_or_default(),
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
