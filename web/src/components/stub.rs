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
                div { class: "mr-empty__icon", aria_hidden: "true", "🍽️" }
                Heading { level: 3, "Nothing here yet" }
                Text { muted: true,
                    "This screen is layout-only for now. The real UI arrives when it's wired to the backend."
                }
            }
        }
    }
}
