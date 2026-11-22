use crate::components::ui::{Button, ButtonVariant, ComboOption, Combobox, Picker, Size};
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::shopping_list::hooks::use_shopping_items::use_shopping_items;
use dioxus::prelude::*;
use domain::shopping_list_item::NewListLine;
use std::collections::HashSet;

/// A search over every ingredient and other item for putting something on the list by
/// hand. A name that matches neither becomes a new item.
#[component]
pub fn AddListItem(busy: bool, on_add: EventHandler<NewListLine>) -> Element {
    let ingredients = use_get_ingredients();
    let mut items = use_shopping_items();
    let mut picker = use_signal(Picker::default);

    let mut options = Vec::new();
    let mut ingredient_ids = HashSet::new();
    if let Some(Ok(list)) = &*ingredients.read() {
        for ingredient in list {
            ingredient_ids.insert(ingredient.id);
            options.push(
                ComboOption::new(ingredient.id, ingredient.name.clone()).with_hint("ingredient"),
            );
        }
    }
    if let Some(Ok(list)) = &*items.read() {
        options.extend(
            list.iter()
                .map(|item| ComboOption::new(item.id, item.name.clone()).with_hint("item")),
        );
    }
    options.sort_by_key(|option| option.label.to_lowercase());

    let current = picker();
    let chosen = match current.picked {
        Some(option) if ingredient_ids.contains(&option.id) => {
            Some(NewListLine::Ingredient(option.id))
        }
        Some(option) => Some(NewListLine::Item(option.label)),
        None => Some(current.query.trim().to_string())
            .filter(|name| !name.is_empty())
            .map(NewListLine::Item),
    };
    let empty = chosen.is_none();

    rsx! {
        form {
            class: "mr-addcustom",
            onsubmit: move |e: FormEvent| {
                e.prevent_default();
                if let (Some(line), false) = (chosen.clone(), busy) {
                    on_add.call(line);
                    picker.set(Picker::default());
                    items.restart();
                }
            },
            Combobox {
                placeholder: "Search ingredients and items, or type a new item".to_string(),
                options,
                picker,
                enterkeyhint: "done".to_string(),
            }
            Button {
                variant: ButtonVariant::Outline,
                size: Size::Md,
                loading: busy,
                disabled: empty,
                submit: true,
                "Add"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_button_submits_the_form() {
        #[component]
        fn Harness() -> Element {
            rsx! {
                AddListItem { busy: false, on_add: |_| {} }
            }
        }

        let mut dom = VirtualDom::new(Harness);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        let button = &html[html.rfind("<button").expect("the form renders a button")..];
        let button = &button[..button.find('>').unwrap()];
        assert!(button.contains(r#"type="submit""#), "{button}");
    }
}
