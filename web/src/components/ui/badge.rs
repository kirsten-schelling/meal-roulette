use dioxus::prelude::*;

/// Semantic color of a badge. Maps to a background/foreground token pair.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Tone {
    Green,
    Amber,
    Red,
    #[default]
    Neutral,
}

impl Tone {
    fn cls(self) -> &'static str {
        match self {
            Tone::Green => "mr-badge--green",
            Tone::Amber => "mr-badge--amber",
            Tone::Red => "mr-badge--red",
            Tone::Neutral => "mr-badge--neutral",
        }
    }
}

#[component]
pub fn Badge(#[props(default)] tone: Tone, children: Element) -> Element {
    let cls = tone.cls();
    rsx! {
        span { class: "mr-badge {cls}", {children} }
    }
}
