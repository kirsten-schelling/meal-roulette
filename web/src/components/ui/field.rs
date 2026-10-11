use dioxus::prelude::*;

/// The shared chrome around a form control: optional label above, optional
/// error message below. Flags the whole field with the danger state while
/// `error` is set. Form controls (`Input`, `Select`, …) wrap themselves in
/// this so the layout stays identical across control types. `required` adds a
/// small "Required" tag beside the label.
#[component]
pub fn Field(
    #[props(optional)] label: Option<String>,
    #[props(optional)] error: Option<String>,
    #[props(default)] required: bool,
    children: Element,
) -> Element {
    let cls = if error.is_some() {
        "mr-field mr-field--error"
    } else {
        "mr-field"
    };

    rsx! {
        div { class: "{cls}",
            if let Some(l) = label.clone() {
                label { class: "mr-label",
                    "{l}"
                    if required {
                        span { class: "mr-label__required", "Required" }
                    }
                }
            }
            {children}
            if let Some(err) = error.clone() {
                span { class: "mr-field__error", "{err}" }
            }
        }
    }
}
