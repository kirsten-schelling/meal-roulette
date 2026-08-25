use crate::components::stub::PageStub;
use dioxus::prelude::*;

#[component]
pub fn PlannedMeals() -> Element {
    rsx! {
        PageStub {
            title: "Planned Meals".to_string(),
            subtitle: "The week the roulette rolled for you.".to_string(),
        }
    }
}
