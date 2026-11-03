use dioxus::prelude::*;

/// An id for a form control, stable across the server render and the browser
/// hydration of the same tree (it comes from the component's scope, not a counter).
/// Pass it to `Field { control_id }` and set it on the control as `id`.
pub fn use_control_id() -> String {
    use_hook(|| format!("mr-ctl-{}", dioxus::core::current_scope_id().0))
}

/// The element id `Field` gives its error message, for `aria-describedby`.
pub fn error_id(control_id: &str) -> String {
    format!("{control_id}-err")
}

/// The shared chrome around a form control: optional label above, optional
/// error message below. Flags the whole field with the danger state while
/// `error` is set. Form controls (`Input`, `Select`, …) wrap themselves in
/// this so the layout stays identical across control types. `required` adds a
/// small "Required" tag beside the label.
///
/// `control_id` is the `id` of the control inside: the label points at it with
/// `for`, and the error gets `{control_id}-err` so the control can name it in
/// `aria-describedby`. Without it the label is plain text (a group of controls).
#[component]
pub fn Field(
    #[props(optional)] label: Option<String>,
    #[props(optional)] error: Option<String>,
    #[props(default)] required: bool,
    #[props(optional)] control_id: Option<String>,
    children: Element,
) -> Element {
    let cls = if error.is_some() {
        "mr-field mr-field--error"
    } else {
        "mr-field"
    };
    let err_id = control_id.as_deref().map(error_id);

    rsx! {
        div { class: "{cls}",
            if let Some(l) = label.clone() {
                label { class: "mr-label", r#for: control_id.clone(),
                    "{l}"
                    if required {
                        span { class: "mr-label__required", "Required" }
                    }
                }
            }
            {children}
            if let Some(err) = error.clone() {
                span { class: "mr-field__error", id: err_id.clone(), role: "alert", "{err}" }
            }
        }
    }
}
