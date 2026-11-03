use crate::components::ui::{ComboOption, Combobox, Picker};
use crate::features::recipe::components::ingredient_line::RecipeLine;
use dioxus::prelude::*;
use domain::ingredient::IngredientEntry;

/// For an imported line: the ingredient it landed on, as a combobox the user can repoint at
/// any library ingredient or a new name. Each resolved choice goes to `on_change` with the
/// line id; the list owner decides whether to apply it.
#[component]
pub fn ImportMatch(
    line: RecipeLine,
    ingredient_options: Vec<ComboOption>,
    on_change: EventHandler<(usize, Picker)>,
    #[props(default)] taken: Vec<String>,
) -> Element {
    let Some(import) = line.import.as_ref() else {
        return rsx! {};
    };
    let id = line.id;
    let phrase = import.phrase.clone();
    let start = match &line.ingredient {
        IngredientEntry::Existing(ing) => {
            let mut p = Picker::default();
            p.pick(ComboOption::new(*ing, line.name.clone()).with_hint(line.unit.clone()));
            p
        }
        IngredientEntry::New { name, .. } => {
            let mut p = Picker::default();
            p.create(name.clone());
            p
        }
    };
    let picker = use_signal(|| start);
    use_effect(move || {
        let p = picker();
        if p.is_resolved() {
            on_change.call((id, p));
        }
    });

    let current = picker();
    let duplicate = current.is_resolved()
        && !current.query.trim().eq_ignore_ascii_case(&line.name)
        && taken.contains(&current.query.trim().to_lowercase());
    let hint = match (&line.ingredient, import.matched) {
        (IngredientEntry::Existing(_), true) => format!("Matched from \"{phrase}\""),
        (IngredientEntry::Existing(_), false) => format!("Chosen for \"{phrase}\""),
        (IngredientEntry::New { .. }, _) => format!("New ingredient for \"{phrase}\""),
    };

    rsx! {
        div { class: "mr-import-match",
            Combobox {
                placeholder: "Search, or type a new one and press Enter…".to_string(),
                options: ingredient_options,
                picker,
            }
            if duplicate {
                span { "Already in this recipe." }
            } else {
                span { "{hint}" }
            }
        }
    }
}
