use crate::components::ui::{Icon, IconButton, IconName, Input, Tone};
use crate::features::pantry::hooks::use_pantry::set_pantry_qty;
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::pantry::PantryLine;
use rust_decimal::Decimal;
use std::str::FromStr;

/// The count typed in a field, if it is a number of zero or more.
pub fn parse_qty(text: &str) -> Option<Decimal> {
    Decimal::from_str(text.trim())
        .ok()
        .filter(|qty| !qty.is_sign_negative())
}

/// One pantry count as a field the user types a correction into; it saves when the
/// field loses focus or Enter is pressed. "Used up" sets the count to 0.
#[component]
pub fn PantryRow(line: PantryLine, on_saved: EventHandler<()>) -> Element {
    let toast = use_toast();
    let saved = line.qty.normalize().to_string();
    let mut text = use_signal(|| saved.clone());
    let mut saving = use_signal(|| false);
    let ingredient_id = line.ingredient_id;

    let mut save = move |qty: Decimal| {
        saving.set(true);
        spawn(async move {
            match set_pantry_qty(ingredient_id, qty).await {
                Ok(line) => {
                    text.set(line.qty.normalize().to_string());
                    on_saved.call(());
                }
                Err(e) => toast.show(
                    Some("Couldn't save the count".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            saving.set(false);
        });
    };

    let commit = {
        let saved = saved.clone();
        move || {
            if text().trim() == saved {
                return;
            }
            match parse_qty(&text()) {
                Some(qty) => save(qty),
                None => {
                    toast.show(
                        Some("Not a count".to_string()),
                        "Type a number of 0 or more.".to_string(),
                        Tone::Red,
                    );
                    text.set(saved.clone());
                }
            }
        }
    };
    let mut commit_on_blur = commit.clone();
    let mut commit_on_enter = commit;

    rsx! {
        div { class: "mr-pantry__row",
            div { class: "mr-pantry__text",
                span { class: "mr-cell-strong", "{line.ingredient}" }
                if !line.unit.is_empty() {
                    span { class: "mr-cooklog__meta", "{line.unit}" }
                }
            }
            div { class: "mr-pantry__count",
                Input {
                    kind: "text".to_string(),
                    inputmode: "decimal".to_string(),
                    enterkeyhint: "done".to_string(),
                    disabled: saving(),
                    value: text(),
                    oninput: move |e: FormEvent| text.set(e.value()),
                    onblur: move |_| commit_on_blur(),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            commit_on_enter();
                        }
                    },
                }
            }
            IconButton {
                label: format!("Mark {} as used up", line.ingredient),
                onclick: move |_| save(Decimal::ZERO),
                Icon { name: IconName::X }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_is_a_number_of_zero_or_more() {
        assert_eq!(parse_qty(" 1.5 "), Some(Decimal::new(15, 1)));
        assert_eq!(parse_qty("0"), Some(Decimal::ZERO));
        assert_eq!(parse_qty("-2"), None);
        assert_eq!(parse_qty("lots"), None);
        assert_eq!(parse_qty(""), None);
    }
}
