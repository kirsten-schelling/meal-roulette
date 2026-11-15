use crate::components::ui::{Button, ButtonVariant, Size};
use dioxus::prelude::*;

/// A one-line form for putting something that is not an ingredient on the list.
/// `on_add` fires with the trimmed text; the page clears the field by resetting `busy`.
#[component]
pub fn AddCustomItem(busy: bool, on_add: EventHandler<String>) -> Element {
    let mut text = use_signal(String::new);
    let trimmed = text().trim().to_string();
    let empty = trimmed.is_empty();

    rsx! {
        form {
            class: "mr-addcustom",
            onsubmit: move |e: FormEvent| {
                e.prevent_default();
                if !empty && !busy {
                    on_add.call(trimmed.clone());
                    text.set(String::new());
                }
            },
            input {
                class: "mr-input",
                r#type: "text",
                value: "{text}",
                placeholder: "Item to add, e.g. bin bags",
                aria_label: "Item to add",
                onmounted: move |e: MountedEvent| {
                    spawn(async move {
                        let _ = e.data().set_focus(true).await;
                    });
                },
                maxlength: "200",
                oninput: move |e: FormEvent| text.set(e.value()),
            }
            Button {
                variant: ButtonVariant::Outline,
                size: Size::Md,
                loading: busy,
                disabled: empty,
                submit: true,
                "Add"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_button_submits_the_form() {
        let html = dioxus_ssr::render_element(rsx! {
            AddCustomItem { busy: false, on_add: |_| {} }
        });

        let button = &html[html.find("<button").expect("the form renders a button")..];
        let button = &button[..button.find('>').unwrap()];
        assert!(button.contains(r#"type="submit""#), "{button}");
    }
}
