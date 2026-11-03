use crate::components::ui::input::focus_at_end;
use crate::components::ui::{Button, ButtonVariant, Icon, IconName, Size};
use dioxus::prelude::*;

/// The resting state of a click-to-edit value: shows `children` with a pencil on hover,
/// and calls `onclick` so the owner can swap in an [`InlineEditor`]. `label` names what
/// gets edited ("Edit cuisine") for screen readers and the hover tooltip.
#[component]
pub fn Editable(label: String, onclick: EventHandler<()>, children: Element) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "mr-editable",
            title: "{label}",
            onclick: move |_| onclick.call(()),
            span { class: "mr-sr-only", "{label}: " }
            span { class: "mr-editable__value", {children} }
            span { class: "mr-editable__icon", aria_hidden: "true",
                Icon { name: IconName::Edit }
            }
        }
    }
}

/// The open state of a click-to-edit value: the editing control (`children`), any error,
/// and Save / Cancel. Enter in the control saves and Escape cancels. Leave `on_save`
/// out for a control that saves on its own (a rating saves on click), so only Cancel
/// shows; turn `enter_saves` off for a control that uses Enter itself (a tag picker
/// adds a tag on Enter).
///
/// It focuses its control as soon as it opens (a text field with the caret after its
/// text), so the tap that opened it is the only one needed before typing.
#[component]
pub fn InlineEditor(
    on_save: Option<EventHandler<()>>,
    on_cancel: EventHandler<()>,
    #[props(default)] saving: bool,
    error: Option<String>,
    #[props(default = true)] enter_saves: bool,
    children: Element,
) -> Element {
    let id = use_hook(|| format!("mr-inline-editor-{}", dioxus::core::current_scope_id().0));
    rsx! {
        div { class: "mr-inline-editor",
            div {
                id: "{id}",
                onmounted: {
                    let id = id.clone();
                    move |_| focus_at_end(&id)
                },
                onkeydown: move |e: KeyboardEvent| match e.key() {
                    Key::Enter if enter_saves => {
                        if let Some(save) = on_save {
                            e.prevent_default();
                            save.call(());
                        }
                    }
                    Key::Escape => on_cancel.call(()),
                    _ => {}
                },
                {children}
            }
            if let Some(msg) = error {
                span { class: "mr-field__error", "{msg}" }
            }
            div { class: "mr-inline-editor__actions",
                if let Some(save) = on_save {
                    Button {
                        size: Size::Sm,
                        loading: saving,
                        onclick: move |_| save.call(()),
                        "Save"
                    }
                }
                Button {
                    size: Size::Sm,
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
            }
        }
    }
}
