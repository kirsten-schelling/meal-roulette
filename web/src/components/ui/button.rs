use dioxus::prelude::*;

/// Visual style of a button. `Solid` is the default (the filled accent one).
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Solid,
    Outline,
    Ghost,
    Danger,
}

impl ButtonVariant {
    fn cls(self) -> &'static str {
        match self {
            ButtonVariant::Solid => "mr-btn--solid",
            ButtonVariant::Outline => "mr-btn--outline",
            ButtonVariant::Ghost => "mr-btn--ghost",
            ButtonVariant::Danger => "mr-btn--danger",
        }
    }
}

/// Shared size scale — also used by other primitives later.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Size {
    Sm,
    #[default]
    Md,
    Lg,
}

impl Size {
    fn btn_cls(self) -> &'static str {
        match self {
            Size::Sm => "mr-btn--sm",
            Size::Md => "mr-btn--md",
            Size::Lg => "mr-btn--lg",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: Size,
    /// Shows a spinner and disables the button.
    #[props(default)]
    loading: bool,
    #[props(default)] full_width: bool,
    #[props(default)] disabled: bool,
    /// Submits the surrounding `<form>`. Off by default, so a button inside a form
    /// doesn't submit it by accident.
    #[props(default)]
    submit: bool,
    /// Optional click handler. Omit for display-only buttons.
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let full = if full_width { "mr-btn--full" } else { "" };
    let class = format!("mr-btn {} {} {}", variant.cls(), size.btn_cls(), full);

    rsx! {
        button {
            class: "{class}",
            r#type: if submit { "submit" } else { "button" },
            disabled: disabled || loading,
            aria_busy: loading.then_some("true"),
            onclick: move |evt| {
                if let Some(cb) = onclick {
                    cb.call(evt);
                }
            },
            // The label stays in the layout while loading (hidden), so the button
            // keeps its width and the spinner sits over it.
            span { class: "mr-btn__label", {children} }
            if loading {
                span { class: "mr-spinner mr-btn__spinner", aria_hidden: "true" }
            }
        }
    }
}
