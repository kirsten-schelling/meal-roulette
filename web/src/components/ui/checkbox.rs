use dioxus::prelude::*;

/// A labeled checkbox. Controlled: pass `checked` and update it in `onchange`,
/// which fires with the new boolean value.
#[component]
pub fn Checkbox(
    checked: bool,
    onchange: EventHandler<bool>,
    #[props(optional)] label: Option<String>,
) -> Element {
    rsx! {
        label { class: "mr-check",
            input {
                r#type: "checkbox",
                checked,
                oninput: move |_| onchange.call(!checked),
            }
            if let Some(l) = label.clone() {
                span { "{l}" }
            }
        }
    }
}
