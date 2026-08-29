use crate::components::ui::{Button, ButtonVariant, Modal};
use dioxus::prelude::*;

/// A small `Modal` preset for yes/no questions. Mount it conditionally, like
/// `Modal` itself. `danger` styles the confirm button for destructive actions;
/// `loading` shows a spinner on it while the action runs.
#[component]
pub fn ConfirmDialog(
    title: String,
    #[props(default = "Confirm".to_string())] confirm_label: String,
    #[props(default)] danger: bool,
    #[props(default)] loading: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
    children: Element,
) -> Element {
    let variant = if danger {
        ButtonVariant::Danger
    } else {
        ButtonVariant::Solid
    };

    rsx! {
        Modal { title: title.clone(), on_close: move |_| on_cancel.call(()),
            {children}
            div { class: "mr-dialog-foot",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                Button {
                    variant,
                    loading,
                    onclick: move |_| on_confirm.call(()),
                    "{confirm_label}"
                }
            }
        }
    }
}
