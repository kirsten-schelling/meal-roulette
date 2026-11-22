use crate::server::api::shopping_item::get_shopping_items;
use dioxus::prelude::*;
use domain::shopping_item::ShoppingItem;

pub use crate::server::api::shopping_item::{
    add_shopping_item, delete_shopping_item, update_shopping_item,
};

/// Every item that is not an ingredient, by name.
pub fn use_shopping_items() -> Resource<Result<Vec<ShoppingItem>, ServerFnError>> {
    use_resource(get_shopping_items)
}
