use crate::components::stub::PageStub;
use dioxus::prelude::*;

#[component]
pub fn Ingredients() -> Element {
    rsx! {
        PageStub {
            title: "Ingredients".to_string(),
            subtitle: "The building blocks your recipes are made from.".to_string(),
        }
    }
}
