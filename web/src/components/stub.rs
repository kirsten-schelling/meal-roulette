//! A placeholder screen used by every section that isn't built yet: a page
//! heading plus an empty state. Swap the `PageStub` in each page for real UI
//! as those sections get wired to the backend.

use crate::components::ui::*;
use dioxus::prelude::*;

#[component]
pub fn PageStub(title: String, subtitle: String) -> Element {
    rsx! {
        div { class: "mr-page",
            header { class: "mr-page__head",
                Heading { level: 1, "{title}" }
                Text { muted: true, "{subtitle}" }
            }
            div { class: "mr-empty",
                div { class: "mr-empty__icon mr-empty__icon--lg", LiftedCloche {} }
                Heading { level: 3, "Nothing here yet" }
                Text { muted: true,
                    "This screen is layout-only for now. The real UI arrives when it's wired to the backend."
                }
            }
        }
    }
}

/// A serving cloche lifted off an empty plate.
#[component]
fn LiftedCloche() -> Element {
    rsx! {
        svg {
            class: "mr-icon",
            view_box: "0 0 64 64",
            fill: "none",
            stroke: "currentColor",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M14 33a18 18 0 0 1 36 0M11 33h42M8 48h48", stroke_width: "2" }
            circle { cx: "32", cy: "12.5", r: "2.5", stroke_width: "2" }
            path { d: "M24 39.5v2M32 39v3M40 39.5v2M14 52h36", stroke_width: "1.5" }
        }
    }
}
