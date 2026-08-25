use crate::server::api::recipe::get_recipes;
use dioxus::prelude::*;
use domain::recipe::Recipe;

pub fn use_get_recipes() -> Resource<Result<Vec<Recipe>, ServerFnError>> {
    use_resource(move || async move {
        let recipes: Result<Vec<Recipe>, ServerFnError> = get_recipes().await;
        recipes
    })
}
