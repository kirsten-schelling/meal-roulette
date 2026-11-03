use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{Badge, Button, ButtonVariant, Icon, IconName, Size, Tone};
use crate::models::{short_date_label, weekday_label};
use dioxus::prelude::*;
use domain::planned_meal::PlannedMealDetail;

/// One dinner of a plan: the day, the recipe, and the two ways to change it.
#[component]
pub fn DayCard(
    meal: PlannedMealDetail,
    /// This is the current day, so the card is highlighted.
    tonight: bool,
    /// This day's re-roll is in flight.
    rerolling: bool,
    /// Another day's re-roll is in flight, so this day's re-roll waits.
    locked: bool,
    on_reroll: EventHandler<()>,
    on_swap: EventHandler<()>,
) -> Element {
    let class = if tonight {
        "mr-daycard mr-daycard--tonight"
    } else {
        "mr-daycard"
    };
    let date = meal.scheduled_date;

    rsx! {
        article { class: "{class}",
            header { class: "mr-daycard__head",
                div { class: "mr-daycard__day",
                    span { class: "mr-daycard__weekday", "{weekday_label(date)}" }
                    span { class: "mr-daycard__date", "{short_date_label(date)}" }
                }
                if tonight {
                    Badge { tone: Tone::Green, "Tonight" }
                }
            }
            h3 { class: "mr-daycard__title", "{meal.title}" }
            div { class: "mr-daycard__meta",
                span { class: "mr-daycard__time", "{meal.time_mins} min" }
                DifficultyBadge { difficulty: meal.difficulty }
                if let Some(cuisine) = meal.cuisine.as_deref() {
                    span { class: "mr-daycard__chip", "{cuisine}" }
                }
                if let Some(protein) = meal.protein.as_deref() {
                    span { class: "mr-daycard__chip", "{protein}" }
                }
            }
            footer { class: "mr-daycard__actions",
                Button {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    loading: rerolling,
                    disabled: locked,
                    onclick: move |_| on_reroll.call(()),
                    Icon { name: IconName::Dice }
                    "Re-roll"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    onclick: move |_| on_swap.call(()),
                    Icon { name: IconName::Replace }
                    "Swap"
                }
            }
        }
    }
}
