//! The Meal Roulette logo: a dinner plate drawn as a roulette wheel, the ball
//! resting in a pocket and the knife and fork laid side by side across it.
//! Colors come from `--mr-*` tokens in the stylesheet, and it is sized by
//! `font-size`.

use dioxus::prelude::*;

#[component]
pub fn BrandMark() -> Element {
    rsx! {
        svg { class: "mr-brandmark", view_box: "0 0 64 64", "aria-hidden": "true",
            circle {
                class: "mr-brandmark__plate",
                cx: "32",
                cy: "32",
                r: "30.5",
                stroke_width: "2",
            }
            // The pocket ring is one circle stroked four times: solid ink, then
            // dash patterns for the red pockets, the green zero, and thin plate-
            // coloured frets. Each dash is a twelfth of the circumference
            // (2π × 25.5 / 12), and the rotation centres the zero at the top.
            g { transform: "rotate(-105 32 32)", fill: "none", stroke_width: "6",
                circle { class: "mr-brandmark__pockets", cx: "32", cy: "32", r: "25.5" }
                circle {
                    class: "mr-brandmark__red",
                    cx: "32",
                    cy: "32",
                    r: "25.5",
                    stroke_dasharray: "13.352 13.352",
                }
                circle {
                    class: "mr-brandmark__zero",
                    cx: "32",
                    cy: "32",
                    r: "25.5",
                    stroke_dasharray: "13.352 146.869",
                }
                circle {
                    class: "mr-brandmark__frets",
                    cx: "32",
                    cy: "32",
                    r: "25.5",
                    stroke_dasharray: "0.9 12.452",
                    stroke_dashoffset: "0.45",
                }
            }
            circle {
                class: "mr-brandmark__plate",
                cx: "32",
                cy: "32",
                r: "22.5",
                stroke_width: "1.5",
            }
            circle {
                class: "mr-brandmark__well",
                cx: "32",
                cy: "32",
                r: "16",
                stroke_width: "1",
            }
            circle {
                class: "mr-brandmark__plate",
                cx: "57.5",
                cy: "32",
                r: "2.2",
                stroke_width: "1",
            }
            g { transform: "rotate(-40 32 32)",
                g { class: "mr-brandmark__line",
                    path {
                        d: "M25.5 14v7M28 14v7M30.5 14v7M25.5 21c0 2.2 1 3.5 2.5 3.5s2.5-1.3 2.5-3.5",
                        stroke_width: "1.6",
                    }
                    path { d: "M28 24.5V50", stroke_width: "2.6" }
                    path { d: "M36.3 33v17", stroke_width: "2.8" }
                }
                path {
                    class: "mr-brandmark__ink",
                    d: "M34.4 33V15.6c0-1.1.9-1.7 1.7-1C38.3 16.4 38.6 23 38.3 33z",
                }
            }
        }
    }
}
