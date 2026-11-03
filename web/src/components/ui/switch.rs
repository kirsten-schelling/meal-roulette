use dioxus::prelude::*;

/// A labeled on/off toggle. Controlled like `Checkbox` — pass `checked`, update
/// it in `onchange` (fires with the new value). Built on a hidden checkbox so
/// it stays keyboard- and screen-reader-friendly.
#[component]
pub fn Switch(
    checked: bool,
    onchange: EventHandler<bool>,
    #[props(optional)] label: Option<String>,
) -> Element {
    rsx! {
        label { class: "mr-switch",
            input {
                r#type: "checkbox",
                role: "switch",
                checked,
                aria_checked: "{checked}",
                oninput: move |_| onchange.call(!checked),
            }
            span { class: "mr-switch__track",
                span { class: "mr-switch__thumb" }
            }
            if let Some(l) = label.clone() {
                span { "{l}" }
            }
        }
    }
}
