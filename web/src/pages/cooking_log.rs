use crate::components::stub::PageStub;
use dioxus::prelude::*;

#[component]
pub fn CookingLog() -> Element {
    rsx! {
        PageStub {
            title: "Cooking Log".to_string(),
            subtitle: "A record of what you've actually cooked.".to_string(),
        }
    }
}
