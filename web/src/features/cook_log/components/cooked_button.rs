use crate::components::ui::{Button, ButtonVariant, Icon, IconName, Text, Tone};
use crate::features::cook_log::hooks::use_cook_log::{
    delete_cook, log_cook, use_get_planned_meal_cook,
};
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use dioxus::prelude::*;
use time::Date;
use uuid::Uuid;

/// Marks a planned dinner as cooked on `cooked_on`, and once it is, undoes that by
/// taking the cook back off the log.
#[component]
pub fn CookedButton(planned_meal_id: Uuid, recipe_id: Uuid, cooked_on: Date) -> Element {
    let mut cook = use_get_planned_meal_cook(planned_meal_id);
    let toast = use_toast();
    let mut saving = use_signal(|| false);

    let logged = match cook() {
        None => {
            return rsx! {
                span { class: "mr-cooked",
                    Button { loading: true, "Cooked it" }
                }
            }
        }
        Some(Err(e)) => {
            return rsx! {
                Text { small: true, muted: true, "Couldn't check the cooking log: {error_message(&e)}" }
            }
        }
        Some(Ok(logged)) => logged,
    };
    let logged_id = logged.as_ref().map(|log| log.id);

    let toggle = move |_| {
        saving.set(true);
        spawn(async move {
            let saved = match logged_id {
                Some(id) => delete_cook(id).await.map(drop),
                None => log_cook(recipe_id, Some(planned_meal_id), cooked_on)
                    .await
                    .map(drop),
            };
            match saved {
                Ok(()) => cook.restart(),
                Err(e) => toast.show(
                    Some("Couldn't update the cooking log".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            saving.set(false);
        });
    };

    rsx! {
        span { class: "mr-cooked",
            if logged_id.is_some() {
                Button {
                    variant: ButtonVariant::Outline,
                    loading: saving(),
                    onclick: toggle,
                    Icon { name: IconName::Check }
                    "Cooked, undo"
                }
            } else {
                Button { loading: saving(), onclick: toggle, "Cooked it" }
            }
        }
    }
}
