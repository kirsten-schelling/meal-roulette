use crate::components::ui::*;
use crate::features::roll::settings::{
    bound_phrase, family_noun, importance_label, RuleLine, IMPORTANCE_LEVELS, MAX_ROLL_DAYS,
};
use dioxus::prelude::*;
use domain::roll::{Bound, Importance, RuleFamily};

/// One rule as a sentence: **Beef** · at least [2] dinners · Must · ×. Every
/// edit reports the whole (count, bound, importance) triple.
#[component]
pub fn RuleRow(
    line: RuleLine,
    on_change: EventHandler<(u32, Bound, Importance)>,
    on_remove: EventHandler<()>,
) -> Element {
    let RuleLine {
        family,
        name,
        times,
        bound,
        importance,
        ..
    } = line;
    let bound_options = vec![
        SelectOption::new("AtLeast", bound_phrase(Bound::AtLeast)),
        SelectOption::new("AtMost", bound_phrase(Bound::AtMost)),
    ];
    let importance_options: Vec<SelectOption> = IMPORTANCE_LEVELS
        .into_iter()
        .map(|level| SelectOption::new(format!("{level:?}"), importance_label(level)))
        .collect();
    // "At most 0" means "none of these"; "at least 0" would ask for nothing.
    let floor = match bound {
        Bound::AtLeast => 1,
        Bound::AtMost => 0,
    };
    let noun = if family == RuleFamily::Ingredient {
        if times == 1 {
            "dinner that uses it"
        } else {
            "dinners that use it"
        }
    } else if times == 1 {
        "dinner"
    } else {
        "dinners"
    };

    rsx! {
        li { class: "mr-rule",
            div { class: "mr-rule__head",
                span { class: "mr-rule__name", "{name}" }
                span { class: "mr-rule__kind", "{family_noun(family)}" }
            }
            div { class: "mr-rule__body",
                Select {
                    compact: true,
                    aria_label: format!("{name}: at least or at most"),
                    value: format!("{bound:?}"),
                    options: bound_options,
                    onchange: move |value: String| {
                        let bound = if value == "AtMost" { Bound::AtMost } else { Bound::AtLeast };
                        let floor = if bound == Bound::AtMost { 0 } else { 1 };
                        on_change.call((times.max(floor), bound, importance));
                    },
                }
                NumberStepper {
                    value: times,
                    onchange: move |n| on_change.call((n, bound, importance)),
                    label: format!("{name}: number of dinners"),
                    min: floor,
                    max: MAX_ROLL_DAYS,
                }
                span { "{noun}" }
                Select {
                    compact: true,
                    aria_label: format!("{name}: how much it matters"),
                    value: format!("{importance:?}"),
                    options: importance_options,
                    onchange: move |value: String| {
                        let level = IMPORTANCE_LEVELS
                            .into_iter()
                            .find(|level| format!("{level:?}") == value)
                            .unwrap_or_default();
                        on_change.call((times, bound, level));
                    },
                }
            }
            IconButton {
                label: format!("Remove the {name} rule"),
                onclick: move |_| on_remove.call(()),
                Icon { name: IconName::X }
            }
        }
    }
}
