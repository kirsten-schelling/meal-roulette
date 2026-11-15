use crate::components::ui::{Chip, Field, NumberStepper, Switch, Text};
use crate::features::recipe::cadence::{gap_words, Cadence, MAX_GAP_WEEKS};
use dioxus::prelude::*;

/// How often a recipe should come round: a preset, or exact weeks behind a switch.
/// 0 weeks on a stepper means no limit that way.
#[component]
pub fn CadencePicker(
    label: Option<String>,
    mut min_weeks: Signal<Option<i16>>,
    mut max_weeks: Signal<Option<i16>>,
) -> Element {
    let preset = Cadence::of(min_weeks(), max_weeks());
    let mut exact = use_signal(|| preset.is_none());

    rsx! {
        Field { label,
            div { class: "mr-cadence__presets", role: "group", aria_label: "How often",
                for cadence in Cadence::ALL {
                    Chip {
                        key: "{cadence.label()}",
                        label: cadence.label().to_string(),
                        active: preset == Some(cadence),
                        onclick: move |_| {
                            let (min, max) = cadence.gaps();
                            min_weeks.set(min);
                            max_weeks.set(max);
                        },
                    }
                }
            }
            Text { small: true, muted: true, "{gap_words(min_weeks(), max_weeks())}" }
            Switch {
                checked: exact(),
                onchange: move |on| exact.set(on),
                label: "Exact weeks".to_string(),
            }
            if exact() {
                div { class: "mr-cadence__exact",
                    span { "Not again for" }
                    NumberStepper {
                        value: stepper_value(min_weeks()),
                        onchange: move |n| min_weeks.set(weeks_from(n)),
                        label: "Weeks before it can come round again".to_string(),
                        max: MAX_GAP_WEEKS,
                    }
                    span { "weeks" }
                }
                div { class: "mr-cadence__exact",
                    span { "Cook at least every" }
                    NumberStepper {
                        value: stepper_value(max_weeks()),
                        onchange: move |n| max_weeks.set(weeks_from(n)),
                        label: "Weeks by which it comes round again".to_string(),
                        max: MAX_GAP_WEEKS,
                    }
                    span { "weeks" }
                }
                Text { small: true, muted: true, "0 means no limit." }
            }
        }
    }
}

fn stepper_value(weeks: Option<i16>) -> u32 {
    weeks.and_then(|w| u32::try_from(w).ok()).unwrap_or(0)
}

fn weeks_from(stepper: u32) -> Option<i16> {
    i16::try_from(stepper).ok().filter(|&w| w > 0)
}
