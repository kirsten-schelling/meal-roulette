use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Dir {
    #[default]
    Column,
    Row,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Align {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}

impl Align {
    fn css(self) -> &'static str {
        match self {
            Align::Stretch => "stretch",
            Align::Start => "flex-start",
            Align::Center => "center",
            Align::End => "flex-end",
        }
    }
}

/// Flexbox layout helper. `gap` is in rem. Direction/gap/align are layout, not
/// theme, so they're applied inline rather than through the token stylesheet.
#[component]
pub fn Stack(
    #[props(default)] direction: Dir,
    #[props(default = 0.75)] gap: f32,
    #[props(default)] align: Align,
    children: Element,
) -> Element {
    let dir = match direction {
        Dir::Row => "row",
        Dir::Column => "column",
    };
    let align = align.css();
    let style = format!("flex-direction:{dir};gap:{gap}rem;align-items:{align};");

    rsx! {
        div { class: "mr-stack", style: "{style}", {children} }
    }
}

/// Vertical stack (column). Convenience wrapper over `Stack`.
#[component]
pub fn VStack(
    #[props(default = 0.75)] gap: f32,
    #[props(default)] align: Align,
    children: Element,
) -> Element {
    rsx! {
        Stack { direction: Dir::Column, gap, align, {children} }
    }
}

/// Horizontal stack (row). Convenience wrapper over `Stack`.
#[component]
pub fn HStack(
    #[props(default = 0.75)] gap: f32,
    #[props(default)] align: Align,
    children: Element,
) -> Element {
    rsx! {
        Stack { direction: Dir::Row, gap, align, {children} }
    }
}
