//! The Meal Roulette logo: a plate on a tile with the roulette ball come to
//! rest on its rim. Colors come from `--mr-*` tokens in the stylesheet, and it
//! is sized by `font-size`.

use dioxus::prelude::*;

#[component]
pub fn BrandMark() -> Element {
    rsx! {
        svg { class: "mr-brandmark", view_box: "0 0 64 64", "aria-hidden": "true",
            rect { class: "mr-brandmark__tile", width: "64", height: "64", rx: "14" }
            circle { class: "mr-brandmark__plate", cx: "32", cy: "32", r: "21" }
            circle {
                class: "mr-brandmark__well",
                cx: "32",
                cy: "32",
                r: "13",
                stroke_width: "3",
            }
            circle {
                class: "mr-brandmark__ball",
                cx: "44",
                cy: "20",
                r: "6.5",
                stroke_width: "3.5",
            }
        }
    }
}
