use crate::components::ui::*;
use crate::features::roll::settings::RollSettings;
use dioxus::prelude::*;
use domain::roll::{DayLimits, DifficultyBand};

fn band_options() -> Vec<SelectOption> {
    let mut options = vec![SelectOption::new("", "Any")];
    options.extend(
        DifficultyBand::ALL
            .into_iter()
            .map(|band| SelectOption::new(format!("{band:?}"), format!("{band:?}"))),
    );
    options
}

fn band_value(band: Option<DifficultyBand>) -> String {
    band.map(|b| format!("{b:?}")).unwrap_or_default()
}

fn parse_band(value: &str) -> Option<DifficultyBand> {
    DifficultyBand::ALL
        .into_iter()
        .find(|band| format!("{band:?}") == value)
}

/// One limit for every day, plus own limits for any single day. Days are
/// numbered from the first rolled day, whatever date the roll starts on.
#[component]
pub fn DayLimitsCard(mut settings: Signal<RollSettings>, day_count: usize) -> Element {
    let current = settings.read().day_limits.clone();

    rsx! {
        VStack { gap: 0.75,
            Text { small: true, muted: true,
                "Cap difficulty and cook time. Applies to every day unless a day has its own."
            }
            Switch {
                checked: current.enabled,
                label: "Limit recipes by day".to_string(),
                onchange: move |on| settings.write().day_limits.enabled = on,
            }
            if current.enabled {
                LimitRow {
                    name: "All days".to_string(),
                    value: current.default,
                    on_change: move |v: DayLimits| settings.write().day_limits.default = v,
                }
                for day in 0..day_count {
                    DayRow {
                        key: "{day}",
                        day,
                        default: current.default,
                        own: current.override_for(day),
                        on_change: move |v: Option<DayLimits>| {
                            settings.write().day_limits.set_override(day, v);
                        },
                    }
                }
            }
        }
    }
}

/// One rolled day: it follows the limits for all days, or has its own.
#[component]
fn DayRow(
    day: usize,
    default: DayLimits,
    own: Option<DayLimits>,
    on_change: EventHandler<Option<DayLimits>>,
) -> Element {
    rsx! {
        VStack { gap: 0.5,
            Switch {
                checked: own.is_some(),
                label: format!("Day {}: own limits", day + 1),
                onchange: move |on: bool| on_change.call(on.then_some(default)),
            }
            if let Some(value) = own {
                LimitRow {
                    name: format!("Day {}", day + 1),
                    value,
                    on_change: move |v: DayLimits| on_change.call(Some(v)),
                }
            }
        }
    }
}

/// The easiest and hardest band a day takes and an optional time cap.
#[component]
fn LimitRow(name: String, value: DayLimits, on_change: EventHandler<DayLimits>) -> Element {
    rsx! {
        div { style: "display:flex; align-items:flex-end; gap:.6rem; flex-wrap:wrap;",
            span { style: "min-width:10rem; padding-bottom:.55rem; font-weight:600;", "{name}" }
            div { style: "width:9rem;",
                Select {
                    label: "Easiest".to_string(),
                    value: band_value(value.min_difficulty),
                    options: band_options(),
                    onchange: move |v: String| {
                        on_change
                            .call(DayLimits {
                                min_difficulty: parse_band(&v),
                                ..value
                            })
                    },
                }
            }
            div { style: "width:9rem;",
                Select {
                    label: "Hardest".to_string(),
                    value: band_value(value.max_difficulty),
                    options: band_options(),
                    onchange: move |v: String| {
                        on_change
                            .call(DayLimits {
                                max_difficulty: parse_band(&v),
                                ..value
                            })
                    },
                }
            }
            div { style: "width:9rem;",
                Input {
                    label: "Max minutes".to_string(),
                    kind: "number".to_string(),
                    value: value.max_time_mins.map(|m| m.to_string()).unwrap_or_default(),
                    placeholder: "No limit".to_string(),
                    oninput: move |e: FormEvent| {
                        on_change
                            .call(DayLimits {
                                max_time_mins: e.value().trim().parse::<u32>().ok().filter(|m| *m > 0),
                                ..value
                            })
                    },
                }
            }
        }
    }
}
