use crate::components::ui::{Select, SelectOption};
use crate::features::shopping_list::hooks::use_shopping_categories::use_shopping_categories;
use dioxus::prelude::*;
use uuid::Uuid;

/// Picks the shopping list category something goes under, or none.
#[component]
pub fn CategorySelect(value: Option<Uuid>, onchange: EventHandler<Option<Uuid>>) -> Element {
    let categories = use_shopping_categories();
    let mut options = vec![SelectOption::new("", "No category")];
    if let Some(Ok(list)) = &*categories.read() {
        options.extend(
            list.iter()
                .map(|c| SelectOption::new(c.id.to_string(), c.name.clone())),
        );
    }

    rsx! {
        Select {
            label: "Shopping category".to_string(),
            value: value.map(|id| id.to_string()).unwrap_or_default(),
            options,
            onchange: move |v: String| onchange.call(v.parse().ok()),
        }
    }
}
