use crate::server::api::shopping_category::get_shopping_categories;
use dioxus::prelude::*;
use domain::shopping_category::ShoppingCategory;

pub use crate::server::api::shopping_category::{
    add_shopping_category, delete_shopping_category, rename_shopping_category,
    reorder_shopping_categories,
};

/// Every shopping category, in list order.
pub fn use_shopping_categories() -> Resource<Result<Vec<ShoppingCategory>, ServerFnError>> {
    use_resource(get_shopping_categories)
}
