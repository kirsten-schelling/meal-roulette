use crate::components::ui::{
    Alert, Button, ButtonVariant, Icon, IconButton, IconName, Modal, Size, Spinner, Tone,
};
use crate::features::shopping_list::hooks::use_shopping_categories::{
    add_shopping_category, delete_shopping_category, rename_shopping_category,
    reorder_shopping_categories,
};
use crate::models::error_message;
use dioxus::prelude::*;
use domain::shopping_category::ShoppingCategory;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Change = Pin<Box<dyn Future<Output = Result<(), ServerFnError>>>>;

/// Add, rename, reorder and delete the categories the shopping list is grouped by.
/// `categories` is the page's fetch; `on_changed` asks the page to fetch again.
/// Mount it conditionally, like `Modal`.
#[component]
pub fn ManageCategoriesDialog(
    categories: Option<Result<Vec<ShoppingCategory>, ServerFnError>>,
    on_changed: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let mut new_name = use_signal(String::new);
    // Row id + draft text while a name is being edited (one at a time).
    let mut editing = use_signal(|| None as Option<(Uuid, String)>);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);

    let mut run = move |change: Change| {
        if busy() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn(async move {
            match change.await {
                Ok(()) => on_changed.call(()),
                Err(e) => error.set(Some(error_message(&e))),
            }
            busy.set(false);
        });
    };

    let mut add = move || {
        let name = new_name().trim().to_string();
        if name.is_empty() {
            return;
        }
        new_name.set(String::new());
        run(Box::pin(async move {
            add_shopping_category(name).await.map(|_| ())
        }));
    };

    let mut save_rename = move |id: Uuid, name: String| {
        editing.set(None);
        run(Box::pin(async move {
            rename_shopping_category(id, name).await.map(|_| ())
        }));
    };

    let ids: Vec<Uuid> = match &categories {
        Some(Ok(list)) => list.iter().map(|c| c.id).collect(),
        _ => Vec::new(),
    };
    let swap = move |at: usize, with: usize| {
        let mut order = ids.clone();
        order.swap(at, with);
        run(Box::pin(async move {
            reorder_shopping_categories(order).await.map(|_| ())
        }));
    };

    rsx! {
        Modal {
            title: "Shopping categories".to_string(),
            on_close: move |_| on_close.call(()),

            div { class: "mr-manage",
                div { class: "mr-manage__content",
                    div { class: "mr-manage__add",
                        input {
                            class: "mr-input",
                            r#type: "text",
                            value: "{new_name}",
                            placeholder: "New category, e.g. Fruit and veg",
                            aria_label: "New category",
                            maxlength: "100",
                            oninput: move |e| new_name.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    add();
                                }
                            },
                        }
                        Button {
                            variant: ButtonVariant::Solid,
                            size: Size::Sm,
                            loading: busy(),
                            disabled: busy() || new_name().trim().is_empty(),
                            onclick: move |_| add(),
                            "Add"
                        }
                    }

                    if let Some(msg) = error() {
                        Alert { tone: Tone::Red, title: "Couldn't change the categories", "{msg}" }
                    }

                    match &categories {
                        None => rsx! {
                            div { class: "mr-manage__state", Spinner {} }
                        },
                        Some(Err(e)) => rsx! {
                            Alert { tone: Tone::Red, title: "Couldn't load the categories", "{error_message(e)}" }
                        },
                        Some(Ok(list)) if list.is_empty() => rsx! {
                            div { class: "mr-manage__state",
                                p { class: "mr-empty-note",
                                    "No categories yet. Add them in the order you walk the shop."
                                }
                            }
                        },
                        Some(Ok(list)) => rsx! {
                            div { class: "mr-managelist mr-scroll",
                                for (at, category) in list.iter().enumerate() {
                                    if let Some((_, draft)) = editing().filter(|(id, _)| *id == category.id) {
                                        div { key: "{category.id}", class: "mr-managelist__row mr-managelist__row--editing",
                                            input {
                                                class: "mr-input",
                                                r#type: "text",
                                                value: "{draft}",
                                                aria_label: "New name",
                                                autofocus: true,
                                                oninput: {
                                                    let id = category.id;
                                                    move |e: FormEvent| editing.set(Some((id, e.value())))
                                                },
                                                onkeydown: {
                                                    let id = category.id;
                                                    let draft = draft.trim().to_string();
                                                    move |e: KeyboardEvent| match e.key() {
                                                        Key::Enter if !draft.is_empty() => save_rename(id, draft.clone()),
                                                        Key::Escape => editing.set(None),
                                                        _ => {}
                                                    }
                                                },
                                            }
                                            IconButton {
                                                label: "Save name".to_string(),
                                                onclick: {
                                                    let id = category.id;
                                                    let draft = draft.trim().to_string();
                                                    move |_| {
                                                        if !draft.is_empty() {
                                                            save_rename(id, draft.clone());
                                                        }
                                                    }
                                                },
                                                Icon { name: IconName::Check }
                                            }
                                            IconButton {
                                                label: "Cancel rename".to_string(),
                                                onclick: move |_| editing.set(None),
                                                Icon { name: IconName::X }
                                            }
                                        }
                                    } else {
                                        div { key: "{category.id}", class: "mr-managelist__row",
                                            span { class: "mr-managelist__name", "{category.name}" }
                                            div { class: "mr-rowactions",
                                                button {
                                                    r#type: "button",
                                                    class: "mr-iconbtn",
                                                    disabled: busy() || at == 0,
                                                    aria_label: "Move {category.name} up",
                                                    onclick: {
                                                        let mut swap = swap.clone();
                                                        move |_| swap(at, at - 1)
                                                    },
                                                    "\u{2191}"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "mr-iconbtn",
                                                    disabled: busy() || at + 1 == list.len(),
                                                    aria_label: "Move {category.name} down",
                                                    onclick: {
                                                        let mut swap = swap.clone();
                                                        move |_| swap(at, at + 1)
                                                    },
                                                    "\u{2193}"
                                                }
                                                IconButton {
                                                    label: "Rename {category.name}".to_string(),
                                                    onclick: {
                                                        let target = (category.id, category.name.clone());
                                                        move |_| editing.set(Some(target.clone()))
                                                    },
                                                    Icon { name: IconName::Edit }
                                                }
                                                IconButton {
                                                    label: "Delete {category.name}".to_string(),
                                                    danger: true,
                                                    onclick: {
                                                        let id = category.id;
                                                        move |_| run(Box::pin(async move {
                                                            delete_shopping_category(id).await.map(|_| ())
                                                        }))
                                                    },
                                                    Icon { name: IconName::X }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
