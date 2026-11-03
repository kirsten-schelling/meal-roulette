//! Keyboard navigation shared by the menus (`Select`, `Combobox`, `SearchBar`):
//! a highlighted-option index that arrow keys move, and the ids that tie the
//! highlighted option to the control through `aria-activedescendant`.

use dioxus::prelude::*;

/// Where the highlight goes after `key`, in a list of `len` options. Anything that
/// isn't a navigation key leaves it where it is.
pub fn next_index(current: Option<usize>, len: usize, key: &Key) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        Key::ArrowDown => Some(current.map_or(0, |i| (i + 1).min(len - 1))),
        Key::ArrowUp => Some(current.map_or(len - 1, |i| i.saturating_sub(1))),
        Key::Home => Some(0),
        Key::End => Some(len - 1),
        _ => current,
    }
}

/// Keys that move the highlight.
pub fn is_nav_key(key: &Key) -> bool {
    matches!(key, Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End)
}

/// The element id of option `i` in the menu whose control id is `base`.
pub fn option_id(base: &str, i: usize) -> String {
    format!("{base}-opt-{i}")
}

/// Scrolls an option into view inside its (scrolling) list.
pub fn reveal(id: &str) {
    let _ = document::eval(&format!(
        "document.getElementById('{id}')?.scrollIntoView({{block:'nearest'}});"
    ));
}

/// `class` for an option row: the selected and highlighted states.
pub fn option_class(selected: bool, highlighted: bool) -> &'static str {
    match (selected, highlighted) {
        (true, true) => "mr-sel__opt mr-sel__opt--active mr-sel__opt--hl",
        (true, false) => "mr-sel__opt mr-sel__opt--active",
        (false, true) => "mr-sel__opt mr-sel__opt--hl",
        (false, false) => "mr-sel__opt",
    }
}
