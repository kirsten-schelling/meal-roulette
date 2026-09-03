use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Orientation {
    #[default]
    Horizontal,
    #[expect(dead_code)]
    Vertical,
}

/// A divider line. Horizontal by default; pass `label` to get a centered
/// caption with a rule on each side (e.g. "— or —" style section breaks).
#[component]
pub fn Separator(
    #[props(default)] orientation: Orientation,
    #[props(optional)] label: Option<String>,
) -> Element {
    match orientation {
        Orientation::Vertical => rsx! {
            div { class: "mr-sep mr-sep--v" }
        },
        Orientation::Horizontal => match label.clone() {
            Some(l) => rsx! {
                div { class: "mr-sep-label",
                    span { "{l}" }
                }
            },
            None => rsx! {
                hr { class: "mr-sep" }
            },
        },
    }
}
