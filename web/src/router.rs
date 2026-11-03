//! The app's navigation map. Each `Route` variant is a URL + the component
//! that renders at it. `#[layout(AppLayout)]` wraps every route below it in the
//! header + sidebar shell.
//!
//! No data or backend calls live here — this is pure navigation.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::layout::AppLayout;
use crate::features::history::page::History;
use crate::features::library::page::Library;
use crate::features::plan::page::PlanDetail;
use crate::features::recipe::detail_page::RecipePage;
use crate::features::recipe::page::Recipes;
use crate::features::roll::page::Roll;
use crate::features::shopping_list::page::{PlanShoppingList, ShoppingList};
use crate::pages::{CookingLog, Home};
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
    #[route("/recipes/:id")]
    RecipePage { id: Uuid },
    #[route("/library")]
    Library {},
    #[route("/shopping-list")]
    ShoppingList {},
    #[route("/shopping-list/:plan_id")]
    PlanShoppingList { plan_id: Uuid },
    #[route("/cooking-log")]
    CookingLog {},
    #[redirect("/planned-meals", || Route::Home {})]
    #[route("/planned-meals/:id")]
    PlanDetail { id: Uuid },
    #[route("/history")]
    History {},
    #[route("/roll")]
    Roll {},
}
