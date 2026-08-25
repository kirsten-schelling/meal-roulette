//! The app's navigation map. Each `Route` variant is a URL + the component
//! that renders at it. `#[layout(AppLayout)]` wraps every route below it in the
//! header + sidebar shell; routes declared *above* it (the gallery) render bare.
//!
//! No data or backend calls live here — this is pure navigation.

use dioxus::prelude::*;

use crate::components::layout::AppLayout;
use crate::features::recipe::page::Recipes;
use crate::pages::{CookingLog, Home, Ingredients, PlannedMeals, ShoppingList};
use crate::showcase::Showcase;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    // Standalone dev route — the component gallery, with no app shell.
    #[route("/showcase")]
    Showcase {},

    // Everything below shares the header + sidebar shell (AppLayout).
    #[layout(AppLayout)]
    #[route("/")]
    Home {},
    #[route("/recipes")]
    Recipes {},
    #[route("/ingredients")]
    Ingredients {},
    #[route("/shopping-list")]
    ShoppingList {},
    #[route("/cooking-log")]
    CookingLog {},
    #[route("/planned-meals")]
    PlannedMeals {},
}
