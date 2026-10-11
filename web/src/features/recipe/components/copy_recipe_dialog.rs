use crate::components::ui::{Alert, Button, ButtonVariant, Input, Modal, Text, Tone};
use crate::features::recipe::hooks::use_recipe::copy_recipe;
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use crate::router::Route;
use dioxus::prelude::*;
use domain::recipe::Recipe;

/// Copies a recipe (details, tags and ingredient lines) under a new title, then opens
/// the copy so it can be tweaked.
#[component]
pub fn CopyRecipeDialog(recipe: Recipe, on_close: EventHandler<()>) -> Element {
    let navigator = use_navigator();
    let toast = use_toast();
    let mut title = use_signal(|| format!("{} (copy)", recipe.title));
    let mut copying = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);

    let blank = title().trim().is_empty();
    let id = recipe.id;
    let mut copy = move || {
        if copying() || title().trim().is_empty() {
            return;
        }
        copying.set(true);
        error.set(None);
        spawn(async move {
            match copy_recipe(id, title()).await {
                Ok(copy) => {
                    toast.show(
                        Some("Copied".to_string()),
                        format!("{} is ready to tweak.", copy.title),
                        Tone::Green,
                    );
                    on_close.call(());
                    navigator.push(Route::RecipePage { id: copy.id });
                }
                Err(e) => {
                    error.set(Some(error_message(&e)));
                    copying.set(false);
                }
            }
        });
    };

    rsx! {
        Modal {
            title: "Make a copy".to_string(),
            on_close: move |_| on_close.call(()),

            div { class: "mr-dialog-form",
                Text { muted: true,
                    "The copy gets the same details, tags and ingredients. Change anything on it afterwards; this recipe stays as it is."
                }
                div {
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            copy();
                        }
                    },
                    Input {
                        label: "Title of the copy".to_string(),
                        value: title(),
                        required: true,
                        autofocus: true,
                        oninput: move |e: FormEvent| title.set(e.value()),
                    }
                }
                if let Some(msg) = error() {
                    Alert { tone: Tone::Red, title: "Couldn't copy the recipe", "{msg}" }
                }
            }
            div { class: "mr-dialog-foot",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Solid,
                    loading: copying(),
                    disabled: blank,
                    onclick: move |_| copy(),
                    "Make copy"
                }
            }
        }
    }
}
