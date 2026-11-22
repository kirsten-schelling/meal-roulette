use crate::components::ui::{Button, ButtonVariant, Modal};
use dioxus::prelude::*;
use domain::shopping_category::ShoppingCategory;
use uuid::Uuid;

/// Choose the category a line's ingredient or item belongs to. `on_pick` fires with the
/// chosen category, or `None` for none. Mount it conditionally, like `Modal`.
#[component]
pub fn CategoryPicker(
    name: String,
    current: Option<Uuid>,
    categories: Vec<ShoppingCategory>,
    on_pick: EventHandler<Option<Uuid>>,
    on_manage: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let choice = |id: Option<Uuid>| {
        if id == current {
            ButtonVariant::Solid
        } else {
            ButtonVariant::Outline
        }
    };

    rsx! {
        Modal {
            title: format!("Category for {name}"),
            on_close: move |_| on_close.call(()),

            div { class: "mr-categorypick",
                for category in categories {
                    Button {
                        key: "{category.id}",
                        variant: choice(Some(category.id)),
                        full_width: true,
                        onclick: move |_| on_pick.call(Some(category.id)),
                        "{category.name}"
                    }
                }
                Button {
                    variant: choice(None),
                    full_width: true,
                    onclick: move |_| on_pick.call(None),
                    "No category"
                }
            }

            div { class: "mr-dialog-foot",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_manage.call(()),
                    "Manage categories\u{2026}"
                }
            }
        }
    }
}
