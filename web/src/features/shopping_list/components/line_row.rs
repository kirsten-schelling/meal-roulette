use crate::models::amount_label;
use dioxus::prelude::*;
use domain::shopping_list_item::ShoppingListLine;

/// "2 x 500 g": whole packs of the ingredient's purchase size, as far as the amount
/// and pack size are known.
fn packs_label(line: &ShoppingListLine) -> String {
    match (line.packs, line.purchase_size) {
        (Some(packs), Some(size)) => format!("{packs} x {size} {}", line.unit),
        (None, Some(size)) => format!("{size} {} packs", line.unit),
        (_, None) => "No pack size".to_string(),
    }
}

/// The muted second line: the amount (and pack count) and the recipes that need it.
fn secondary(line: &ShoppingListLine) -> String {
    if line.ingredient_id.is_none() {
        return "Added by you".to_string();
    }
    let mut parts = vec![amount_label(line.qty, &line.unit), packs_label(line)];
    if line.used_in.is_empty() {
        parts.push("no longer on the plan".to_string());
    } else {
        parts.push(format!("for: {}", line.used_in.join(", ")));
    }
    parts.join(" · ")
}

/// One list line: a tick target, the name with its muted detail line, and an
/// action slot. `on_tick` is `None` for a line that cannot be ticked (asleep).
#[component]
pub fn LineRow(
    line: ShoppingListLine,
    checked: bool,
    on_tick: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let asleep = line.sleep_until.is_some();
    let name_class = if asleep {
        "mr-linerow__name mr-dim"
    } else if checked {
        "mr-linerow__name mr-struck"
    } else {
        "mr-linerow__name"
    };
    let detail_class = if asleep {
        "mr-linerow__detail mr-dim"
    } else {
        "mr-linerow__detail"
    };
    let detail = secondary(&line);

    rsx! {
        li { class: "mr-linerow",
            if let Some(on_tick) = on_tick {
                label { class: "mr-linerow__tick",
                    input {
                        class: "mr-buycheck",
                        r#type: "checkbox",
                        checked,
                        aria_label: "Bought {line.ingredient}",
                        onchange: move |_| on_tick.call(!checked),
                    }
                }
            }
            div { class: "mr-linerow__main",
                span { class: "{name_class}",
                    "{line.ingredient}"
                    if let Some(note) = &line.note {
                        span { class: "mr-linenote", "{note}" }
                    }
                }
                span { class: "{detail_class}", "{detail}" }
            }
            div { class: "mr-linerow__action", {children} }
        }
    }
}
