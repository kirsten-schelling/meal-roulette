use dioxus::prelude::*;

mod components;
mod models;
mod server;

use components::add_recipe_form::AddRecipeForm;
use components::recipe_table::RecipeTable;
use server::list_recipes;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Loads on the server during SSR (the BFF read), then hydrates.
    let mut recipes = use_server_future(list_recipes)?;
    let list = recipes().and_then(|r| r.ok()).unwrap_or_default();

    rsx! {
        document::Stylesheet { href: "https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css" }

        main { class: "container",
            hgroup {
                h1 { "This week's recipes" }
                p { "{list.len()} in your library · Dioxus fullstack (#[server] BFF)" }
            }

            RecipeTable { recipes: list }
            AddRecipeForm { on_added: move |_| recipes.restart() }
        }
    }
}
