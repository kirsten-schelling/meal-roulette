use crate::features::recipe::hooks::use_recipe_view::RecipeView;
use dioxus::prelude::*;

/// Cards or table. Both buttons stay in the tab order; `aria-pressed` marks the
/// current one.
#[component]
pub fn ViewToggle(view: RecipeView, on_change: EventHandler<RecipeView>) -> Element {
    let options = [(RecipeView::Cards, "Cards"), (RecipeView::Table, "Table")];

    rsx! {
        div { class: "mr-viewtoggle", role: "group", aria_label: "Recipe view",
            for (option , label) in options {
                button {
                    key: "{label}",
                    r#type: "button",
                    class: if option == view { "mr-viewtoggle__btn mr-viewtoggle__btn--active" } else { "mr-viewtoggle__btn" },
                    aria_pressed: option == view,
                    onclick: move |_| on_change.call(option),
                    "{label}"
                }
            }
        }
    }
}
