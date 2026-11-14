//! Route targets outside a feature. Pages compose the `ui` primitives and feature
//! components; they hold no business logic.

pub mod home;
//pub mod recipes;

pub use home::Home;
// pub use recipes::Recipes;
