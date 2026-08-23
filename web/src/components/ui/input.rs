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
) -> Element {
    let field_cls = if error.is_some() {
        "mr-field mr-field--error"
    } else {
        "mr-field"
    };

    rsx! {
        div { class: "{field_cls}",
            if let Some(l) = label.clone() {
                label { class: "mr-label", "{l}" }
            }
            input {
                class: "mr-input",
                r#type: "{kind}",
                value: "{value}",
                placeholder: placeholder.clone().unwrap_or_default(),
                oninput: move |e| oninput.call(e),
            }
            if let Some(err) = error.clone() {
                span { class: "mr-field__error", "{err}" }
            }
        }
    }
}
