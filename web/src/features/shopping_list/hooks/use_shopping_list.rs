use crate::server::api::shopping_list::{get_latest_shopping_list, get_shopping_list};
use dioxus::prelude::*;
use domain::shopping_list::ShoppingListForPlan;
use uuid::Uuid;

pub use crate::server::api::shopping_list::{
    add_covered_item, add_list_line, clear_bought_items, ensure_shopping_list,
    generate_shopping_list, remove_custom_item, set_item_purchased, set_line_category,
    sleep_ingredients, wake_ingredient,
};

/// A plan and its shopping list: the plan `plan_id` names, or the latest
/// active plan when it is `None`. `Ok(None)` means there is no plan at all.
pub fn use_get_shopping_list(
    plan_id: ReadSignal<Option<Uuid>>,
) -> Resource<Result<Option<ShoppingListForPlan>, ServerFnError>> {
    use_resource(move || {
        let plan_id = plan_id();
        async move {
            match plan_id {
                Some(id) => get_shopping_list(id).await.map(Some),
                None => get_latest_shopping_list().await,
            }
        }
    })
}
