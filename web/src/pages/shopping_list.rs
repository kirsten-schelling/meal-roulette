use crate::components::stub::PageStub;
use dioxus::prelude::*;

#[component]
pub fn ShoppingList() -> Element {
    rsx! {
        PageStub {
            title: "Shopping List".to_string(),
            subtitle: "Everything you need to buy for the week's planned meals.".to_string(),
        }
    }
}
