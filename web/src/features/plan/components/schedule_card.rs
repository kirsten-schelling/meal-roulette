use crate::components::ui::{Button, Card, Input, Text};
use crate::models::parse_iso_date;
use dioxus::prelude::*;
use time::Date;

/// Puts a rolled plan's dinners on consecutive dates from a chosen start date, in roll
/// order. Individual dinners can then be moved in the table below.
#[component]
pub fn ScheduleCard(
    /// The date the plan was last laid out from, to prefill the picker.
    start_date: Option<Date>,
    /// How many of the plan's dinners have no date.
    unscheduled: usize,
    busy: bool,
    on_schedule: EventHandler<Date>,
) -> Element {
    let mut start = use_signal(|| start_date.map(|d| d.to_string()).unwrap_or_default());
    let chosen = parse_iso_date(&start());
    let (header, action) = if unscheduled > 0 {
        ("Put on days", "Put on days")
    } else {
        ("Move the week", "Move the week")
    };

    rsx! {
        Card { header: header.to_string(),
            div { class: "mr-schedule",
                div { class: "mr-schedule__field",
                    Input {
                        label: "Day 1 is on".to_string(),
                        kind: "date".to_string(),
                        value: start(),
                        oninput: move |e: FormEvent| start.set(e.value()),
                    }
                }
                Button {
                    loading: busy,
                    disabled: chosen.is_none(),
                    onclick: move |_| {
                        if let Some(date) = chosen {
                            on_schedule.call(date);
                        }
                    },
                    "{action}"
                }
            }
            Text { small: true, muted: true,
                "The dinners land on consecutive days in roll order. Then move any one to another date, or leave them off the calendar."
            }
        }
    }
}
