use crate::features::recipe::components::ingredient_line::RecipeLine;
use dioxus::prelude::*;

/// For an imported line: which library ingredient it landed on, with a one-click switch to
/// adding the page's wording as a new ingredient instead (or back to the library match).
#[component]
pub fn ImportMatch(
    line: RecipeLine,
    on_switch: EventHandler<usize>,
    #[props(default)] blocked: bool,
) -> Element {
    let Some(import) = line.import.as_ref() else {
        return rsx! {};
    };
    let id = line.id;
    let phrase = import.phrase.clone();
    let alternative = import.suggestion.as_ref().map(|s| s.line.name.clone());

    rsx! {
        div { class: "mr-import-match",
            if import.matched {
                span { "Matched: {line.name}" }
                button {
                    r#type: "button",
                    class: "mr-ing-line__edit",
                    disabled: blocked,
                    onclick: move |_| on_switch.call(id),
                    "Add '{phrase}' as new"
                }
            } else if let Some(name) = alternative {
                span { "Adding '{phrase}' as new" }
                button {
                    r#type: "button",
                    class: "mr-ing-line__edit",
                    disabled: blocked,
                    onclick: move |_| on_switch.call(id),
                    "Use {name} instead"
                }
            }
        }
    }
}
