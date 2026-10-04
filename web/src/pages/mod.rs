//! Route targets — one module per page. Pages compose the `ui` primitives and
//! the shared `PageStub`; they hold no business logic yet.

//mod stub;

pub mod cooking_log;
pub mod home;
//pub mod recipes;
pub mod shopping_list;

pub use cooking_log::CookingLog;
pub use home::Home;
// pub use recipes::Recipes;
pub use shopping_list::ShoppingList;

// pub(crate) use stub::PageStub;
