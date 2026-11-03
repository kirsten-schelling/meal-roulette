use crate::components::difficulty_badge::DifficultyBadge;
use crate::components::ui::{Badge, Button, ButtonVariant, Icon, IconName, Size, Tone};
use crate::models::{parse_iso_date, short_date_label, weekday_label};
use dioxus::prelude::*;
use domain::planned_meal::PlannedMealDetail;
use time::Date;

/// One dinner of a plan: the day (or its roll position while it has no date), the recipe,
/// and the ways to change it. The card can be dragged onto another dated card to take that
/// day.
#[component]
pub fn DayCard(
    meal: PlannedMealDetail,
    /// This is the current day, so the card is highlighted.
    tonight: bool,
    /// This day's re-roll is in flight.
    rerolling: bool,
    /// Another day's re-roll is in flight, so this day's re-roll waits.
    locked: bool,
    /// A date change or a whole-plan scheduling is in flight, so the date field waits.
    moving: bool,
    /// Another card is being dragged, so this one shows whether it accepts the drop.
    drop_ready: bool,
    on_reroll: EventHandler<()>,
    on_swap: EventHandler<()>,
    /// The date field changed; `None` takes the dinner off the calendar.
    on_move: EventHandler<Option<Date>>,
    on_drag_start: EventHandler<()>,
    on_drag_end: EventHandler<()>,
    /// Another card was dropped here.
    on_drop: EventHandler<()>,
) -> Element {
    let date = meal.scheduled_date;
    let class = match (tonight, drop_ready && date.is_some()) {
        (true, true) => "mr-daycard mr-daycard--tonight mr-daycard--over",
        (true, false) => "mr-daycard mr-daycard--tonight",
        (false, true) => "mr-daycard mr-daycard--over",
        (false, false) => "mr-daycard",
    };
    let (day, day_sub) = match date {
        Some(date) => (weekday_label(date), short_date_label(date)),
        None => (format!("Day {}", meal.day_index), "No date yet".to_string()),
    };
    let value = date.map(|d| d.to_string()).unwrap_or_default();
    let date_label = format!("Date for {}", meal.title);

    rsx! {
        article {
            class: "{class}",
            draggable: "true",
            ondragstart: move |_| on_drag_start.call(()),
            ondragend: move |_| on_drag_end.call(()),
            ondragover: move |e: DragEvent| {
                if drop_ready && date.is_some() {
                    e.prevent_default();
                }
            },
            ondrop: move |e: DragEvent| {
                e.prevent_default();
                if date.is_some() {
                    on_drop.call(());
                }
            },
            header { class: "mr-daycard__head",
                div { class: "mr-daycard__day",
                    span { class: "mr-daycard__weekday", "{day}" }
                    span { class: "mr-daycard__date", "{day_sub}" }
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
            input {
                class: "mr-input mr-datepick",
                r#type: "date",
                aria_label: "{date_label}",
                value: "{value}",
                disabled: moving,
                onchange: move |e: FormEvent| {
                    let typed = e.value();
                    if typed.trim().is_empty() {
                        on_move.call(None);
                    } else if let Some(date) = parse_iso_date(&typed) {
                        on_move.call(Some(date));
                    }
                },
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
