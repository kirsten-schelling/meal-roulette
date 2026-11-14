use crate::components::ui::{Alert, Button, ButtonVariant, Input, Size, Spinner, Text, Tone};
use crate::features::recipe::hooks::use_recipe::import_recipe_from_url;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::recipe_import::RecipeImport;

/// "Import from a link": fetches a recipe page on the server and hands what it found to
/// `on_import`. Nothing is saved here; the owner fills its form so the user can review it.
/// Collapsed until opened, so it is not mistaken for the title box. Text that is not a link
/// goes to `on_title` when the owner has no title yet (`has_title`), else is refused.
#[component]
pub fn ImportFromLink(
    on_import: EventHandler<RecipeImport>,
    on_title: EventHandler<String>,
    has_title: bool,
) -> Element {
    let mut open = use_signal(|| false);
    let mut url = use_signal(String::new);
    let mut hint = use_signal(|| None as Option<String>);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None as Option<String>);
    let mut imported_from = use_signal(|| None as Option<String>);

    let can_import = !loading() && !url().trim().is_empty();
    let mut import = move || {
        if loading() || !can_import {
            return;
        }
        let link = url().trim().to_string();
        if !is_web_link(&link) {
            if has_title {
                hint.set(Some("That doesn't look like a link".to_string()));
            } else {
                on_title.call(link);
                url.set(String::new());
                open.set(false);
            }
            return;
        }
        spawn(async move {
            loading.set(true);
            error.set(None);
            match import_recipe_from_url(link).await {
                Ok(recipe) => {
                    imported_from.set(Some(recipe.source_url.clone()));
                    on_import.call(recipe);
                    url.set(String::new());
                    open.set(false);
                }
                Err(e) => error.set(Some(error_message(&e))),
            }
            loading.set(false);
        });
    };

    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            onclick: move |_| open.toggle(),
            "Import from a link"
        }
        if open() {
            div { class: "mr-inline-add",
                Input {
                    label: "Recipe page link (not the title)".to_string(),
                    kind: "url".to_string(),
                    inputmode: "url".to_string(),
                    enterkeyhint: "go".to_string(),
                    autocomplete: "off".to_string(),
                    value: url(),
                    oninput: move |e: FormEvent| {
                        url.set(e.value());
                        hint.set(None);
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            import();
                        }
                    },
                    error: hint(),
                    placeholder: "Paste a recipe page address…".to_string(),
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    disabled: !can_import,
                    onclick: move |_| import(),
                    if loading() {
                        Spinner {}
                    } else {
                        "Import"
                    }
                }
            }
        }
        if let Some(msg) = error() {
            Alert { tone: Tone::Red, title: "Couldn't import that page", "{msg}" }
        }
        if let Some(source) = imported_from() {
            Text { muted: true, small: true,
                "Imported from {source}. Check everything below before saving; the main protein still needs picking."
            }
        }
    }
}

/// Whether `value` is an http(s) address rather than something typed in the wrong box.
fn is_web_link(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"));
    rest.is_some_and(|r| !r.is_empty() && !r.contains(char::is_whitespace))
}

#[cfg(test)]
mod tests {
    use super::is_web_link;

    #[test]
    fn accepts_http_and_https_links() {
        assert!(is_web_link("https://example.com/pad-thai"));
        assert!(is_web_link("HTTP://example.com"));
    }

    #[test]
    fn rejects_text_that_is_not_a_link() {
        assert!(!is_web_link("Pad Thai"));
        assert!(!is_web_link("example.com/pad-thai"));
        assert!(!is_web_link("https://"));
        assert!(!is_web_link("https://example.com pad thai"));
        assert!(!is_web_link("ftp://example.com"));
    }
}
