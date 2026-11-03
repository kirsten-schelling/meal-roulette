use crate::components::ui::{Button, ButtonVariant, Modal, Text};
use crate::router::Route;
use dioxus::prelude::*;
use domain::usage::{Reference, ReferenceKind};

/// Shown when a delete was refused because something still uses the value: lists what does,
/// each linking to where it can be fixed, and — when `cascade_warning` is given — offers an
/// explicit "Delete anyway" that spells out what it will remove. Without a warning the value
/// can only be freed by changing those records first. Mount it conditionally, like `Modal`.
#[component]
pub fn BlockedDeleteDialog(
    name: String,
    references: Vec<Reference>,
    cascade_warning: Option<String>,
    #[props(default)] busy: bool,
    on_cascade: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        Modal { title: format!("Can't delete “{name}” yet"), on_close: move |_| on_close.call(()),
            Text { "These still use it. Open one to change it there:" }
            ul { class: "mr-refs mr-scroll",
                for reference in references.iter() {
                    li { key: "{reference.id}", class: "mr-refs__item",
                        ReferenceLink { reference: reference.clone() }
                        span { class: "mr-refs__kind", "{kind_label(reference.kind)}" }
                    }
                }
            }
            if let Some(warning) = cascade_warning.clone() {
                p { class: "mr-refs__warning", "{warning}" }
            }
            div { class: "mr-dialog-foot",
                Button {
                    variant: ButtonVariant::Ghost,
                    disabled: busy,
                    onclick: move |_| on_close.call(()),
                    "Keep it"
                }
                if cascade_warning.is_some() {
                    Button {
                        variant: ButtonVariant::Danger,
                        loading: busy,
                        onclick: move |_| on_cascade.call(()),
                        "Delete anyway"
                    }
                }
            }
        }
    }
}

fn kind_label(kind: ReferenceKind) -> &'static str {
    match kind {
        ReferenceKind::Recipe => "Recipe",
        ReferenceKind::Ingredient => "Ingredient",
        ReferenceKind::ShoppingList => "Shopping list",
    }
}

#[component]
fn ReferenceLink(reference: Reference) -> Element {
    let name = reference.name;
    match reference.kind {
        ReferenceKind::Recipe => rsx! {
            Link { to: Route::RecipePage { id: reference.id }, class: "mr-refs__link", "{name}" }
        },
        ReferenceKind::Ingredient => rsx! {
            Link { to: Route::Library {}, class: "mr-refs__link", "{name}" }
        },
        ReferenceKind::ShoppingList => rsx! {
            Link { to: Route::ShoppingList {}, class: "mr-refs__link", "{name}" }
        },
    }
}
