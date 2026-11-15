//! The colour-palette choice, a second axis beside light and dark. The choice
//! lives in `localStorage` and is applied as `data-palette` on `<html>`; with
//! no attribute the stylesheet's default palette, Sunny kitchen, applies.

use crate::hooks::use_theme::SYNC_THEME_COLOR;
use dioxus::prelude::*;

const STORAGE_KEY: &str = "mr-palette";

/// Runs in `<head>` before first paint so a saved palette does not flash the
/// default first. A stored value with no palette block in the stylesheet
/// falls back to the default there.
pub const EARLY_SCRIPT: &str = "try{var p=localStorage.getItem('mr-palette');if(p){document.documentElement.setAttribute('data-palette',p)}}catch(e){}";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaletteChoice {
    Sunny,
    Warm,
    Herb,
    Paprika,
    Paper,
    Oven,
    Dusk,
}

impl PaletteChoice {
    pub const ALL: [PaletteChoice; 7] = [
        PaletteChoice::Sunny,
        PaletteChoice::Warm,
        PaletteChoice::Herb,
        PaletteChoice::Paprika,
        PaletteChoice::Paper,
        PaletteChoice::Oven,
        PaletteChoice::Dusk,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PaletteChoice::Sunny => "Sunny kitchen",
            PaletteChoice::Warm => "Warm",
            PaletteChoice::Herb => "Fresh herb",
            PaletteChoice::Paprika => "Paprika",
            PaletteChoice::Paper => "Paper and ink",
            PaletteChoice::Oven => "Oven glow",
            PaletteChoice::Dusk => "Dusk rose",
        }
    }

    /// The `data-palette` value: names the palette's block in the stylesheet
    /// and is what `localStorage` holds.
    pub fn key(self) -> &'static str {
        match self {
            PaletteChoice::Sunny => "sunny",
            PaletteChoice::Warm => "warm",
            PaletteChoice::Herb => "herb",
            PaletteChoice::Paprika => "paprika",
            PaletteChoice::Paper => "paper",
            PaletteChoice::Oven => "oven",
            PaletteChoice::Dusk => "dusk",
        }
    }

    fn from_stored(value: &str) -> PaletteChoice {
        PaletteChoice::ALL
            .into_iter()
            .find(|choice| choice.key() == value)
            .unwrap_or(PaletteChoice::Sunny)
    }

    fn apply_script(self) -> String {
        if self == PaletteChoice::Sunny {
            return format!(
                "try{{localStorage.removeItem('{STORAGE_KEY}')}}catch(e){{}}\
                 document.documentElement.removeAttribute('data-palette');"
            );
        }
        let key = self.key();
        format!(
            "try{{localStorage.setItem('{STORAGE_KEY}','{key}')}}catch(e){{}}\
             document.documentElement.setAttribute('data-palette','{key}');"
        )
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    choice: Signal<PaletteChoice>,
}

impl Palette {
    pub fn choice(&self) -> PaletteChoice {
        (self.choice)()
    }

    pub fn set(&mut self, choice: PaletteChoice) {
        self.choice.set(choice);
        let _ = document::eval(&choice.apply_script());
        let _ = document::eval(SYNC_THEME_COLOR);
    }
}

/// Reads the saved choice once the app is running in the browser. The
/// attribute itself is already in place from `EARLY_SCRIPT`; this only syncs
/// the control with it.
pub fn use_palette() -> Palette {
    let mut choice = use_signal(|| PaletteChoice::Sunny);

    use_effect(move || {
        spawn(async move {
            let script = format!(
                "try{{return localStorage.getItem('{STORAGE_KEY}')}}catch(e){{return null}}"
            );
            if let Ok(value) = document::eval(&script).await {
                choice.set(PaletteChoice::from_stored(value.as_str().unwrap_or("")));
            }
        });
    });

    Palette { choice }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette_block<'a>(css: &'a str, key: &str) -> &'a str {
        let selector = format!("[data-palette=\"{key}\"]");
        let start = css
            .find(&selector)
            .unwrap_or_else(|| panic!("no palette block for {key}"));
        let open = start + css[start..].find('{').unwrap();
        let close = open + css[open..].find('}').unwrap();
        &css[open + 1..close]
    }

    fn colour(block: &str, name: &str) -> [f64; 3] {
        let declaration = format!("--mr-{name}:");
        let at = block
            .find(&declaration)
            .unwrap_or_else(|| panic!("no {name}"))
            + declaration.len();
        let hex = block[at..].trim_start();
        let channel = |i: usize| {
            f64::from(u8::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap()) / 255.0
        };
        [channel(0), channel(1), channel(2)]
    }

    fn luminance([r, g, b]: [f64; 3]) -> f64 {
        let linear = |c: f64| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    }

    fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn every_palette_meets_contrast_targets_for_text_badges_and_controls() {
        let css = include_str!("../../assets/primitives.css");
        let pairs = [
            ("text", "bg", 4.5),
            ("text-muted", "bg", 4.5),
            ("text-muted", "surface", 4.5),
            ("accent-contrast", "accent", 4.5),
            ("accent-text", "bg", 4.5),
            ("accent-text", "surface", 4.5),
            ("danger", "surface", 4.5),
            ("bg", "danger", 4.5),
            ("warn", "surface", 3.0),
            ("good", "surface", 3.0),
            ("green-fg", "green-bg", 4.5),
            ("amber-fg", "amber-bg", 4.5),
            ("red-fg", "red-bg", 4.5),
            ("neutral-fg", "neutral-bg", 4.5),
        ];
        for palette in PaletteChoice::ALL {
            let block = palette_block(css, palette.key());
            for scheme in ["light", "dark"] {
                for (fg, bg, min) in pairs {
                    let ratio = contrast(
                        colour(block, &format!("{scheme}-{fg}")),
                        colour(block, &format!("{scheme}-{bg}")),
                    );
                    assert!(
                        ratio >= min,
                        "{} {scheme}: {fg} on {bg} is {ratio:.2}",
                        palette.key()
                    );
                }
            }
        }
    }

    #[test]
    fn phone_status_bar_colour_is_the_top_bar_colour() {
        let css = include_str!("../../assets/primitives.css");
        let phone = &css[css.find("@media (max-width: 720px)").unwrap()..];
        let topbar = &phone[phone.find(".mr-topbar {").unwrap()..];
        let topbar = &topbar[..topbar.find('}').unwrap()];
        assert!(topbar.contains("background: var(--mr-chrome);"));
        assert!(phone.contains("--mr-chrome: var(--mr-surface);"));
        assert!(css.contains("--mr-chrome:     var(--mr-bg);"));
    }

    #[test]
    fn stored_value_round_trips_and_unknown_falls_back_to_sunny() {
        for palette in PaletteChoice::ALL {
            assert!(PaletteChoice::from_stored(palette.key()) == palette);
        }
        assert!(PaletteChoice::from_stored("") == PaletteChoice::Sunny);
        assert!(PaletteChoice::from_stored("neon") == PaletteChoice::Sunny);
    }
}
