//! Full-screen cooking view of a recipe: large type, ingredients that check off with a
//! tap, and the screen kept awake. Recipes have no method steps yet, so it shows
//! ingredients only.

use crate::components::ui::modal::FOCUS_SCRIPT;
use crate::components::ui::{Button, ButtonVariant, Icon, IconName, Text};
use crate::hooks::use_wake_lock::use_wake_lock;
use crate::models::amount_label;
use dioxus::prelude::*;
use domain::recipe_ingredient::RecipeIngredient;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn CookMode(
    title: String,
    ingredients: Vec<RecipeIngredient>,
    on_exit: EventHandler<()>,
) -> Element {
    use_wake_lock();
    let id = use_hook(|| format!("mr-cook-{}", dioxus::core::current_scope_id().0));
    let script_id = id.clone();
    use_effect(move || {
        let _ = document::eval(&FOCUS_SCRIPT.replace("__ID__", &script_id));
    });
    let mut checked = use_signal(HashSet::<Uuid>::new);
    let total = ingredients.len();
    let done = checked.read().len();

    rsx! {
        div {
            id: "{id}",
            class: "mr-cook",
            tabindex: "-1",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    on_exit.call(());
                }
            },
            role: "dialog",
            aria_modal: "true",
            aria_label: "Cooking {title}",
            div { class: "mr-cook__bar",
                div { class: "mr-cook__titles",
                    h2 { class: "mr-cook__title", "{title}" }
                    Text { muted: true, "{done} of {total} ingredients ready" }
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| on_exit.call(()),
                    Icon { name: IconName::X }
                    "Exit cook mode"
                }
            }
            if ingredients.is_empty() {
                p { class: "mr-cook__empty", "This recipe has no ingredients yet." }
            } else {
                ul { class: "mr-cook__list",
                    for line in ingredients {
                        CookItem {
                            key: "{line.id}",
                            line: line.clone(),
                            done: checked.read().contains(&line.id),
                            on_toggle: move |_| {
                                let mut set = checked.write();
                                if !set.remove(&line.id) {
                                    set.insert(line.id);
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CookItem(line: RecipeIngredient, done: bool, on_toggle: EventHandler<()>) -> Element {
    let amount = amount_label(line.qty, &line.unit);
    let mut detail = match line.note.as_deref().map(str::trim) {
        Some(note) if !note.is_empty() => format!("{amount} · {note}"),
        _ => amount,
    };
    if line.is_optional {
        detail.push_str(" · optional");
    }

    rsx! {
        li {
            button {
                r#type: "button",
                class: if done { "mr-cook__item mr-cook__item--done" } else { "mr-cook__item" },
                aria_pressed: done,
                onclick: move |_| on_toggle.call(()),
                span { class: "mr-cook__tick", aria_hidden: "true",
                    if done {
                        Icon { name: IconName::Check }
                    }
                }
                span { class: "mr-cook__text",
                    span { class: "mr-cook__name", "{line.ingredient_name}" }
                    span { class: "mr-cook__detail", "{detail}" }
                }
            }
        }
    }
}
