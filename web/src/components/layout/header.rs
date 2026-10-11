//! Top bar: the brand (links home) on the left, global actions on the right.
//! Right now the only action is the light/dark toggle; account / settings
//! controls can join it later.

use crate::components::ui::*;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Header(
    /// Current theme state, reflected by the toggle.
    dark: bool,
    /// Fired with the new value when the user flips the toggle.
    on_toggle: EventHandler<bool>,
) -> Element {
    rsx! {
        header { class: "mr-header",
            Link { class: "mr-brand", to: Route::Home {},
                BrandMark {}
                span { class: "mr-brand__name", "Meal Roulette" }
            }
            div { class: "mr-header__actions",
                Switch {
                    checked: dark,
                    onchange: move |v| on_toggle.call(v),
                    label: "Dark".to_string(),
                }
            }
        }
    }
}
