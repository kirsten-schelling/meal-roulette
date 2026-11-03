//! The colour-theme choice: follow the OS, or force light or dark. The choice
//! lives in `localStorage` and is applied as `data-theme` on `<html>`; with no
//! attribute the stylesheet follows `prefers-color-scheme`.

use dioxus::prelude::*;

const STORAGE_KEY: &str = "mr-theme";

/// Runs in `<head>` before first paint so a saved override does not flash the
/// OS theme first.
pub const EARLY_SCRIPT: &str = "try{var t=localStorage.getItem('mr-theme');if(t==='light'||t==='dark'){document.documentElement.setAttribute('data-theme',t)}}catch(e){}";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 3] = [ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark];

    pub fn label(self) -> &'static str {
        match self {
            ThemeChoice::System => "System",
            ThemeChoice::Light => "Light",
            ThemeChoice::Dark => "Dark",
        }
    }

    fn from_stored(value: &str) -> ThemeChoice {
        match value {
            "light" => ThemeChoice::Light,
            "dark" => ThemeChoice::Dark,
            _ => ThemeChoice::System,
        }
    }

    fn apply_script(self) -> String {
        let value = match self {
            ThemeChoice::System => {
                return format!(
                    "try{{localStorage.removeItem('{STORAGE_KEY}')}}catch(e){{}}\
                     document.documentElement.removeAttribute('data-theme');"
                );
            }
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
        };
        format!(
            "try{{localStorage.setItem('{STORAGE_KEY}','{value}')}}catch(e){{}}\
             document.documentElement.setAttribute('data-theme','{value}');"
        )
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    choice: Signal<ThemeChoice>,
}

impl Theme {
    pub fn choice(&self) -> ThemeChoice {
        (self.choice)()
    }

    pub fn set(&mut self, choice: ThemeChoice) {
        self.choice.set(choice);
        let _ = document::eval(&choice.apply_script());
    }
}

/// Reads the saved choice once the app is running in the browser. The
/// attribute itself is already in place from `EARLY_SCRIPT`; this only syncs
/// the control with it.
pub fn use_theme() -> Theme {
    let mut choice = use_signal(|| ThemeChoice::System);

    use_effect(move || {
        spawn(async move {
            let script = format!(
                "try{{return localStorage.getItem('{STORAGE_KEY}')}}catch(e){{return null}}"
            );
            if let Ok(value) = document::eval(&script).await {
                choice.set(ThemeChoice::from_stored(value.as_str().unwrap_or("")));
            }
        });
    });

    Theme { choice }
}
