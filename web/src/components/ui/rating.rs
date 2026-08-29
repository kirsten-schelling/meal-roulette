use crate::components::ui::Field;
use dioxus::prelude::*;

/// Colors a rating's dots by position — green/amber/red — so the ends of the
/// scale read at a glance. Pick the direction by what a high value means.
#[derive(Clone, Copy, PartialEq)]
pub enum RatingScale {
    /// High values are good (healthiness): 1 red … 5 green.
    HighIsGood,
    /// High values are bad (difficulty): 1 green … 5 red.
    HighIsBad,
}

impl RatingScale {
    fn dot_cls(self, n: i16, max: i16) -> &'static str {
        // Every dot gets its own step on the bad→good ramp, so neighbours
        // never share a color.
        const RAMP: [&str; 5] = [
            "mr-rating__dot mr-rating__dot--r",
            "mr-rating__dot mr-rating__dot--o",
            "mr-rating__dot mr-rating__dot--a",
            "mr-rating__dot mr-rating__dot--l",
            "mr-rating__dot mr-rating__dot--g",
        ];
        let step = if max > 1 {
            ((n - 1) * 4 / (max - 1)) as usize
        } else {
            4
        };
        match self {
            RatingScale::HighIsGood => RAMP[step],
            RatingScale::HighIsBad => RAMP[4 - step],
        }
    }
}

/// A 1–`max` dot-scale input (difficulty, healthiness, …). Click a dot to set
/// the value. When `on_clear` is provided, clicking the active dot clears the
/// rating — pass it for optional ratings, omit it for required ones. `scale`
/// colors the dots by position; without it they take the accent color.
#[component]
pub fn Rating(
    #[props(optional)] label: Option<String>,
    value: Option<i16>,
    onchange: EventHandler<i16>,
    on_clear: Option<EventHandler<()>>,
    scale: Option<RatingScale>,
    #[props(default = 5)] max: i16,
) -> Element {
    let aria = label.clone().unwrap_or_else(|| "Rating".to_string());

    rsx! {
        Field { label: label.clone(),
            // DOM order is reversed (max..1) and flex-direction row-reverse, so
            // the CSS sibling combinator can preview-fill the dots *before* the
            // hovered one — see .mr-rating in primitives.css.
            div { class: "mr-rating", role: "radiogroup", aria_label: "{aria}",
                for n in (1..=max).rev() {
                    {
                        let filled = value.is_some_and(|v| v >= n);
                        let current = value == Some(n);
                        let base = match scale {
                            Some(s) => s.dot_cls(n, max),
                            None => "mr-rating__dot",
                        };
                        let dot_cls = if filled {
                            format!("{base} mr-rating__dot--on")
                        } else {
                            base.to_string()
                        };
                        rsx! {
                            button {
                                key: "{n}",
                                r#type: "button",
                                class: "{dot_cls}",
                                role: "radio",
                                aria_checked: current,
                                aria_label: "{n} of {max}",
                                onclick: move |_| {
                                    if current {
                                        if let Some(cb) = &on_clear {
                                            cb.call(());
                                            return;
                                        }
                                    }
                                    onchange.call(n);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
