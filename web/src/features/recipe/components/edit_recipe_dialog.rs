use crate::components::ui::{
    Alert, Button, ButtonVariant, ConfirmDialog, Modal, ReasonTip, Text, Tone,
};
use crate::features::library::components::{ManageListsDialog, ManageTab};
use crate::features::recipe::components::recipe_fields::{
    use_recipe_catalogs, use_recipe_form, RecipeFields,
};
use crate::features::recipe::hooks::use_recipe::update_recipe;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::recipe::RecipeOverview;

/// Edits a saved recipe's details (title, lookups, tags, ratings, time, servings). Its
/// ingredient lines are edited on the recipe page itself.
#[component]
pub fn EditRecipeDialog(
    overview: RecipeOverview,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let id = overview.recipe.id;
    let form = use_recipe_form(Some(overview));
    let catalogs = use_recipe_catalogs();
    let mut managing = use_signal(|| None as Option<ManageTab>);
    let mut confirm_discard = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);

    let missing = form.missing();
    let dirty = form.is_dirty();
    let mut request_close = move || {
        if dirty {
            confirm_discard.set(true);
        } else {
            on_close.call(());
        }
    };
    let save = move |_| {
        let Some(update) = form.update() else {
            return;
        };
        if saving() {
            return;
        }
        saving.set(true);
        error.set(None);
        spawn(async move {
            match update_recipe(id, update).await {
                Ok(_) => on_saved.call(()),
                Err(e) => error.set(Some(error_message(&e))),
            }
            saving.set(false);
        });
    };

    rsx! {
        Modal {
            wide: true,
            overflow_visible: true,
            title: "Edit recipe details".to_string(),
            on_close: move |_| request_close(),

            div { class: "mr-recipe-dialog",
                div { class: "mr-recipe-dialog__content",
                    RecipeFields {
                        form,
                        cuisine_options: catalogs.cuisine_options(),
                        protein_options: catalogs.protein_options(),
                        tag_options: catalogs.tag_options(),
                        on_manage: move |tab: ManageTab| managing.set(Some(tab)),
                    }
                    if let Some(msg) = error() {
                        Alert { tone: Tone::Red, title: "Couldn't save the changes", "{msg}" }
                    }
                }
                div { class: "mr-dialog-foot",
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| request_close(),
                        "Cancel"
                    }
                    ReasonTip { reasons: missing.clone(),
                        Button {
                            variant: ButtonVariant::Solid,
                            loading: saving(),
                            disabled: !missing.is_empty() || !dirty,
                            onclick: save,
                            "Save changes"
                        }
                    }
                }
            }
        }

        if let Some(tab) = managing() {
            ManageListsDialog {
                initial_tab: tab,
                on_close: move |_| {
                    managing.set(None);
                    catalogs.restart();
                },
            }
        }

        if confirm_discard() {
            ConfirmDialog {
                title: "Discard your changes?".to_string(),
                confirm_label: "Discard".to_string(),
                danger: true,
                on_confirm: move |_| on_close.call(()),
                on_cancel: move |_| confirm_discard.set(false),
                Text { "The recipe keeps its saved details." }
            }
        }
    }
}
