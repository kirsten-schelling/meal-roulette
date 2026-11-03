use crate::components::ui::{Alert, Button, ButtonVariant, Input, Spinner, Text, Tone};
use crate::features::recipe::hooks::use_recipe::import_recipe_from_url;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::recipe_import::RecipeImport;

/// "Import from a link": fetches a recipe page on the server and hands what it found to
/// `on_import`. Nothing is saved here; the owner fills its form so the user can review it.
#[component]
pub fn ImportFromLink(on_import: EventHandler<RecipeImport>) -> Element {
    let mut url = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);
    let mut imported_from = use_signal(|| None as Option<String>);

    let can_import = !loading() && !url().trim().is_empty();
    let import = move |_| {
        if loading() {
            return;
        }
        let link = url().trim().to_string();
        spawn(async move {
            loading.set(true);
            error.set(None);
            match import_recipe_from_url(link).await {
                Ok(recipe) => {
                    imported_from.set(Some(recipe.source_url.clone()));
                    on_import.call(recipe);
                }
                Err(e) => error.set(Some(error_message(&e))),
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "mr-inline-add",
            Input {
                label: "Import from a link".to_string(),
                kind: "url".to_string(),
                value: url(),
                oninput: move |e: FormEvent| url.set(e.value()),
                placeholder: "Paste a recipe page address…".to_string(),
            }
            Button {
                variant: ButtonVariant::Ghost,
                disabled: !can_import,
                onclick: import,
                if loading() {
                    Spinner {}
                } else {
                    "Import"
                }
            }
        }
        if let Some(msg) = error() {
            Alert { tone: Tone::Red, title: "Couldn't import that page", "{msg}" }
        }
        if let Some(source) = imported_from() {
            Text { muted: true, small: true,
                "Imported from {source}. Check everything below before saving; the main protein still needs picking."
            }
        }
    }
}
