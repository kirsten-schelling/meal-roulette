use crate::components::ui::{Badge, Tone};
use crate::models::{amount_label, measure_label};
use dioxus::prelude::*;
use domain::shopping_list_item::ShoppingListLine;

/// "2 x 500 g": whole packs of the ingredient's purchase size, as far as the amount
/// and pack size are known.
fn packs_label(line: &ShoppingListLine) -> String {
    match (line.packs, line.purchase_size) {
        (Some(packs), Some(size)) => format!("{packs} x {}", measure_label(size, &line.unit)),
        (None, Some(size)) => format!("{} packs", measure_label(size, &line.unit)),
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

/// One list line: a tick target (the box and the text beside it), and an
/// action slot. `on_tick` is `None` for a line that cannot be ticked (asleep).
/// `selected` marks a line queued to be slept.
#[component]
pub fn LineRow(
    line: ShoppingListLine,
    checked: bool,
    on_tick: Option<EventHandler<bool>>,
    #[props(default)] selected: bool,
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
    let row_class = if selected {
        "mr-linerow mr-linerow--selected"
    } else {
        "mr-linerow"
    };
    let detail = secondary(&line);
    let tick_label = if checked {
        format!("Mark {} as not bought", line.ingredient)
    } else {
        format!("Mark {} as bought", line.ingredient)
    };

    let main = rsx! {
        div { class: "mr-linerow__main",
            span { class: "{name_class}",
                "{line.ingredient}"
                if let Some(note) = &line.note {
                    span { class: "mr-linenote", "{note}" }
                }
                if selected {
                    Badge { tone: Tone::Amber, "Will sleep" }
                }
            }
            span { class: "{detail_class}", "{detail}" }
        }
    };

    rsx! {
        li { class: "{row_class}",
            if let Some(on_tick) = on_tick {
                label { class: "mr-linerow__hit mr-linerow__hit--tick",
                    span { class: "mr-linerow__tick",
                        input {
                            class: "mr-buycheck",
                            r#type: "checkbox",
                            checked,
                            aria_label: "{tick_label}",
                            onchange: move |_| on_tick.call(!checked),
                        }
                    }
                    {main}
                }
            } else {
                div { class: "mr-linerow__hit", {main} }
            }
            div { class: "mr-linerow__action", {children} }
        }
    }
}
