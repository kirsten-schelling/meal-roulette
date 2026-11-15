//! The phone navigation: a slim top bar with a menu button that slides a drawer
//! in from the left. The drawer holds the same links as the sidebar plus the
//! palette and theme controls. Hidden above 720px, where the sidebar shows instead.
//!
//! The drawer is always in the DOM and hidden with `visibility`, so it can
//! animate and stays out of the tab order while closed. It behaves as a modal
//! dialog while open: focus moves in, Tab stays inside, and Escape, the scrim,
//! a link click or a swipe to the left close it and hand focus back to the button.

use crate::components::layout::sidebar::NavLinks;
use crate::components::layout::{NavEntry, PalettePicker, ThemeToggle, VersionLine};
use crate::components::ui::{BrandMark, Icon, IconButton, IconName};
use crate::router::Route;
use dioxus::prelude::*;

const DRAWER_ID: &str = "mr-drawer";
const MENU_BTN_ID: &str = "mr-menu-btn";

/// Installs the Tab trap and the swipe-left gesture once. A swipe clicks the
/// scrim so it takes the same path as every other way of closing.
const SETUP_SCRIPT: &str = r#"
const drawer = document.getElementById("mr-drawer");
if (drawer && !drawer.dataset.ready) {
  drawer.dataset.ready = "1";
  const sel = 'a[href],button:not([disabled]),input:not([disabled]),[tabindex]:not([tabindex="-1"])';
  drawer.addEventListener('keydown', (e) => {
    if (e.key !== 'Tab') return;
    const items = Array.from(drawer.querySelectorAll(sel)).filter((el) => el.getClientRects().length > 0);
    if (items.length === 0) { e.preventDefault(); return; }
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
    else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
  });
  let x0 = null, y0 = null;
  drawer.addEventListener('touchstart', (e) => {
    x0 = e.touches[0].clientX; y0 = e.touches[0].clientY;
  }, { passive: true });
  drawer.addEventListener('touchend', (e) => {
    if (x0 === null) return;
    const dx = e.changedTouches[0].clientX - x0;
    const dy = e.changedTouches[0].clientY - y0;
    x0 = null;
    if (dx < -60 && Math.abs(dx) > Math.abs(dy) * 1.5) {
      const scrim = document.querySelector('.mr-scrim');
      if (scrim) scrim.click();
    }
  }, { passive: true });
}
"#;

const FOCUS_IN_SCRIPT: &str = r#"
setTimeout(() => {
  const drawer = document.getElementById("mr-drawer");
  const target = drawer && drawer.querySelector('a[href]');
  if (target) target.focus();
}, 50);
"#;

const FOCUS_BUTTON_SCRIPT: &str = r#"
const btn = document.getElementById("mr-menu-btn");
if (btn) btn.focus();
"#;

#[component]
pub fn MobileNav(primary: Vec<NavEntry>, manage: Vec<NavEntry>) -> Element {
    let mut open = use_signal(|| false);

    use_effect(|| {
        let _ = document::eval(SETUP_SCRIPT);
    });

    let mut close = move || {
        if open() {
            open.set(false);
            let _ = document::eval(FOCUS_BUTTON_SCRIPT);
        }
    };

    let state = if open() { " mr-drawer--open" } else { "" };

    rsx! {
        header { class: "mr-topbar",
            button {
                id: MENU_BTN_ID,
                r#type: "button",
                class: "mr-iconbtn mr-topbar__menu",
                aria_label: "Open menu",
                aria_expanded: "{open()}",
                aria_controls: DRAWER_ID,
                onclick: move |_| {
                    open.set(true);
                    let _ = document::eval(FOCUS_IN_SCRIPT);
                },
                Icon { name: IconName::Menu }
            }
            Link { class: "mr-brand", to: Route::Home {},
                BrandMark {}
                span { class: "mr-brand__name", "Meal Roulette" }
            }
        }
        div {
            class: "mr-scrim{state}",
            aria_hidden: "true",
            onclick: move |_| close(),
        }
        aside {
            id: DRAWER_ID,
            class: "mr-drawer{state}",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Menu",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    close();
                }
            },
            div { class: "mr-drawer__head",
                Link { class: "mr-brand", to: Route::Home {}, onclick: move |_| close(),
                    BrandMark {}
                    span { class: "mr-brand__name", "Meal Roulette" }
                }
                IconButton {
                    label: "Close menu".to_string(),
                    onclick: move |_| close(),
                    Icon { name: IconName::X }
                }
            }
            nav { class: "mr-nav", aria_label: "Sections",
                NavLinks { entries: primary, on_navigate: move |_| close() }
                div { class: "mr-nav__group", "Manage" }
                NavLinks { entries: manage, quiet: true, on_navigate: move |_| close() }
            }
            div { class: "mr-drawer__foot",
                PalettePicker {}
                ThemeToggle {}
                VersionLine {}
            }
        }
    }
}
