//! Landing page: the welcome + the roulette. Everything here is visual only —
//! the "Spin" button just toggles a decorative wheel animation so the page
//! feels alive. The real roll-a-week-of-meals logic gets wired in later.

use crate::components::ui::*;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    // Placeholder interaction: flips the wheel's spin animation on/off.
    let mut spinning = use_signal(|| false);
    let wheel_cls = if spinning() {
        "mr-wheel mr-wheel--spin"
    } else {
        "mr-wheel"
    };
    let label = if spinning() { "Stop" } else { "Spin the wheel" };

    rsx! {
        section { class: "mr-hero",
            div { class: "mr-hero__stage",
                div { class: "mr-hero__pointer", aria_hidden: "true" }
                div { class: "{wheel_cls}", aria_hidden: "true",
                    div { class: "mr-wheel__hub" }
                }
            }

            Heading { level: 1, "What's for dinner?" }
            div { class: "mr-hero__sub",
                Text { muted: true,
                    "Spin to roll a week of meals from your recipes, then turn it into a shopping list in one click."
                }
            }

            Button {
                variant: ButtonVariant::Solid,
                size: Size::Lg,
                onclick: move |_| spinning.toggle(),
                "{label}"
            }
            Text { small: true, muted: true, "Layout preview — the spin isn't wired to anything yet." }
        }
    }
}
