use crate::components::ui::{Alert, Button, Card, ComboOption, Combobox, Input, Picker, Tone};
use crate::features::cook_log::hooks::use_cook_log::log_cook;
use crate::features::recipe::hooks::use_recipe::use_get_recipes;
use crate::hooks::use_toast::use_toast;
use crate::hooks::use_today::use_today;
use crate::models::{error_message, parse_iso_date};
use dioxus::prelude::*;

/// Logs a cook that wasn't on a plan: pick a recipe and the day, which starts at today.
#[component]
pub fn LogCookCard(on_logged: EventHandler<()>) -> Element {
    let recipes = use_get_recipes();
    let today = use_today();
    let toast = use_toast();
    let mut recipe = use_signal(Picker::default);
    // The browser answers "today" after the first render, so the field shows today until
    // a date is typed rather than being seeded once.
    let mut typed_date = use_signal(|| None as Option<String>);
    let mut saving = use_signal(|| false);

    let date_text = typed_date()
        .or_else(|| today().map(|date| date.to_string()))
        .unwrap_or_default();
    let cooked_on = parse_iso_date(&date_text);
    let recipe_id = recipe.read().picked.as_ref().map(|option| option.id);
    let options: Vec<ComboOption> = match recipes() {
        Some(Ok(list)) => list
            .iter()
            .map(|r| ComboOption::new(r.id, r.title.clone()))
            .collect(),
        _ => Vec::new(),
    };

    let submit = move |_| {
        let (Some(recipe_id), Some(cooked_on)) = (recipe_id, cooked_on) else {
            return;
        };
        saving.set(true);
        spawn(async move {
            match log_cook(recipe_id, None, cooked_on).await {
                Ok(_) => {
                    recipe.set(Picker::default());
                    on_logged.call(());
                }
                Err(e) => toast.show(
                    Some("Couldn't log the cook".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            saving.set(false);
        });
    };

    rsx! {
        Card { header: "Log a cook".to_string(),
            if let Some(Err(e)) = recipes() {
                Alert { tone: Tone::Red, title: "Couldn't load recipes", "{error_message(&e)}" }
            }
            div { class: "mr-logcook",
                Combobox {
                    label: "Recipe".to_string(),
                    placeholder: "Search recipes…".to_string(),
                    options,
                    picker: recipe,
                    can_create: false,
                }
                Input {
                    label: "Cooked on".to_string(),
                    kind: "date".to_string(),
                    value: date_text,
                    oninput: move |e: FormEvent| typed_date.set(Some(e.value())),
                }
                Button {
                    full_width: true,
                    loading: saving(),
                    disabled: recipe_id.is_none() || cooked_on.is_none(),
                    onclick: submit,
                    "Log it"
                }
            }
        }
    }
}
