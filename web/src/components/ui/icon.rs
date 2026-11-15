//! Line icons on a 24×24 grid, stroked in `currentColor` and sized at `1em`,
//! so an icon takes the text color and font size of whatever it sits in.
//!
//! Paths from Tabler Icons (https://tabler.io/icons),
//! Copyright (c) 2020-2024 Paweł Kuna, MIT License.

use dioxus::prelude::*;

/// Icons are named for their shape, not for where they are used.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IconName {
    Archive,
    ArrowLeft,
    CalendarWeek,
    Carrot,
    Check,
    Checklist,
    ChefHat,
    ChevronRight,
    Dice,
    Edit,
    History,
    Menu,
    Pin,
    Plus,
    Replace,
    Star,
    X,
}

impl IconName {
    fn path(self) -> &'static str {
        match self {
            IconName::Archive => {
                "M3 6a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2z \
                 M5 8v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-10 M10 12l4 0"
            }
            IconName::ArrowLeft => "M5 12l14 0 M5 12l6 6 M5 12l6 -6",
            IconName::CalendarWeek => {
                "M4 7a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2v-12z \
                 M16 3v4 M8 3v4 M4 11h16 M7 14h.013 M10.01 14h.005 M13.01 14h.005 M16.015 14h.005 \
                 M13.015 17h.005 M7.01 17h.005 M10.01 17h.005"
            }
            IconName::Carrot => {
                "M3 21s9.834 -3.489 12.684 -6.34a4.487 4.487 0 0 0 0 -6.344a4.483 4.483 0 0 0 -6.342 0\
                 c-2.86 2.861 -6.347 12.689 -6.347 12.689z M9 13l-1.5 -1.5 M16 14l-2 -2 \
                 M22 8s-1.14 -2 -3 -2c-1.406 0 -3 2 -3 2s1.14 2 3 2s3 -2 3 -2z \
                 M16 2s-2 1.14 -2 3s2 3 2 3s2 -1.577 2 -3c0 -1.86 -2 -3 -2 -3z"
            }
            IconName::Check => "M5 12l5 5l10 -10",
            IconName::Checklist => {
                "M9.615 20h-2.615a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2h8a2 2 0 0 1 2 2v8 \
                 M14 19l2 2l4 -4 M9 8h4 M9 12h2"
            }
            IconName::ChefHat => {
                "M12 3c1.918 0 3.52 1.35 3.91 3.151a4 4 0 0 1 2.09 7.723l0 7.126h-12v-7.126\
                 a4 4 0 1 1 2.092 -7.723a4 4 0 0 1 3.908 -3.151z M6.161 17.009l11.839 -.009"
            }
            IconName::ChevronRight => "M9 6l6 6l-6 6",
            // Tabler draws the pips as filled circles; here each is a tiny
            // circle whose 2-unit stroke fills it in, so one path covers it all.
            IconName::Dice => {
                "M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2z \
                 M8 8.5a.5 .5 0 1 0 1 0a.5 .5 0 1 0 -1 0 \
                 M11.5 12a.5 .5 0 1 0 1 0a.5 .5 0 1 0 -1 0 \
                 M15 15.5a.5 .5 0 1 0 1 0a.5 .5 0 1 0 -1 0"
            }
            IconName::Edit => {
                "M7 7h-1a2 2 0 0 0 -2 2v9a2 2 0 0 0 2 2h9a2 2 0 0 0 2 -2v-1 \
                 M20.385 6.585a2.1 2.1 0 0 0 -2.97 -2.97l-8.415 8.385v3h3l8.385 -8.415z M16 5l3 3"
            }
            IconName::History => "M12 8l0 4l2 2 M3.05 11a9 9 0 1 1 .5 4m-.5 5v-5h5",
            IconName::Menu => "M4 6l16 0 M4 12l16 0 M4 18l16 0",
            IconName::Pin => {
                "M15 4.5l-4 4l-4 1.5l-1.5 1.5l7 7l1.5 -1.5l1.5 -4l4 -4 M9 15l-4.5 4.5 M14.5 4l5.5 5.5"
            }
            IconName::Plus => "M12 5l0 14 M5 12l14 0",
            IconName::Replace => {
                "M3 4a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z \
                 M15 16a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z \
                 M21 11v-3a2 2 0 0 0 -2 -2h-6l3 3m0 -6l-3 3 M3 13v3a2 2 0 0 0 2 2h6l-3 -3m0 6l3 -3"
            }
            IconName::Star => {
                "M12 17.75l-6.172 3.245l1.179 -6.873l-5 -4.867l6.9 -1l3.086 -6.253l3.086 6.253l6.9 1\
                 l-5 4.867l1.179 6.873z"
            }
            IconName::X => "M18 6l-12 12 M6 6l12 12",
        }
    }
}

/// Decorative by design (`aria-hidden`): the control or text around an icon
/// carries its meaning, e.g. `IconButton`'s `label`.
#[component]
pub fn Icon(name: IconName) -> Element {
    rsx! {
        svg {
            class: "mr-icon",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: name.path() }
        }
    }
}
