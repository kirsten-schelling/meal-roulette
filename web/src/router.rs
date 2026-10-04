//! The app's navigation map. Each `Route` variant is a URL + the component
//! that renders at it. `#[layout(AppLayout)]` wraps every route below it in the
//! header + sidebar shell.
//!
//! No data or backend calls live here — this is pure navigation.

use dioxus::prelude::*;

use crate::components::layout::AppLayout;
use crate::features::library::page::Library;
use crate::features::plan::page::PlannedMeals;
use crate::features::recipe::page::Recipes;
use crate::features::roll::page::Roll;
use crate::pages::{CookingLog, Home, ShoppingList};
use crate::showcase::Showcase;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    // Everything below shares the header + sidebar shell (AppLayout).
    #[layout(AppLayout)]
    #[route("/")]
    Home {},
    // Dev route — the component gallery.
    #[route("/showcase")]
    Showcase {},
    #[route("/recipes")]
    Recipes {},
    #[route("/library")]
    Library {},
    #[route("/shopping-list")]
    ShoppingList {},
    #[route("/cooking-log")]
    CookingLog {},
    #[route("/planned-meals")]
    PlannedMeals {},
    #[route("/roll")]
    Roll {},
}
