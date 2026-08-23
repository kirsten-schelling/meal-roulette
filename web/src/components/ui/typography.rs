use dioxus::prelude::*;

/// A serif heading. `level` picks the tag/size (1–3); defaults to 2.
#[component]
pub fn Heading(#[props(default = 2)] level: u8, children: Element) -> Element {
    match level {
        1 => rsx! { h1 { class: "mr-h1", {children} } },
        3 => rsx! { h3 { class: "mr-h3", {children} } },
        _ => rsx! { h2 { class: "mr-h2", {children} } },
    }
}

/// Body text. `muted` dims it to the secondary color; `small` shrinks it.
#[component]
pub fn Text(
    #[props(default)] muted: bool,
    #[props(default)] small: bool,
    children: Element,
) -> Element {
    let mut class = String::from("mr-text");
    if muted {
        class.push_str(" mr-text--muted");
    }
    if small {
        class.push_str(" mr-text--sm");
    }
    rsx! {
        p { class: "{class}", {children} }
    }
}
