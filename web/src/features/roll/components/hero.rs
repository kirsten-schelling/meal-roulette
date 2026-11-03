use crate::components::ui::*;
use crate::features::roll::settings::MAX_ROLL_DAYS;
use dioxus::prelude::*;

/// The top of the roll page: the wheel, how many dinners to plan, and the
/// preset that sets the rest. The Roll button follows in `RollBar`.
#[component]
pub fn RollHero(
    days: ReadSignal<u32>,
    on_days: EventHandler<u32>,
    spinning: bool,
    children: Element,
) -> Element {
    let wheel = if spinning {
        "mr-wheel mr-wheel--sm mr-wheel--spin"
    } else {
        "mr-wheel mr-wheel--sm"
    };

    rsx! {
        header { class: "mr-roll-hero",
            div { class: "mr-hero__stage",
                div { class: "mr-hero__pointer" }
                div { class: "{wheel}", "aria-hidden": "true",
                    div { class: "mr-wheel__hub" }
                }
            }
            Heading { level: 1, "Roll a week" }
            div { class: "mr-roll-plan",
                span { "Plan" }
                NumberStepper {
                    value: days,
                    onchange: move |n| on_days.call(n),
                    label: "Number of dinners".to_string(),
                    min: 1,
                    max: MAX_ROLL_DAYS,
                }
                span {
                    if days() == 1 {
                        "dinner"
                    } else {
                        "dinners"
                    }
                }
            }
            {children}
        }
    }
}

/// The Roll button and the line under it. On a phone it sticks to the bottom
/// of the screen so it is always in reach.
#[component]
pub fn RollBar(
    rolling: bool,
    disabled: bool,
    on_roll: EventHandler<()>,
    #[props(optional)] note: Option<String>,
    #[props(default)] note_warns: bool,
) -> Element {
    let note_cls = if note_warns {
        "mr-rollbar__note mr-rollbar__note--warn"
    } else {
        "mr-rollbar__note"
    };

    rsx! {
        div { class: "mr-rollbar",
            Button {
                variant: ButtonVariant::Solid,
                size: Size::Lg,
                loading: rolling,
                disabled,
                onclick: move |_| on_roll.call(()),
                "Roll"
            }
            if let Some(text) = note.clone() {
                p { class: "{note_cls}", "{text}" }
            }
        }
    }
}
