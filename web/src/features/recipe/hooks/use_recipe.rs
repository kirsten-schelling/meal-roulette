use crate::server::api::recipe::{add_recipe, get_recipes};
use dioxus::prelude::*;
use domain::recipe::{NewRecipe, Recipe};

#[derive(Clone, Copy)]
pub struct AddRecipe {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Recipe, ServerFnError>>>,
}

pub fn use_get_recipes() -> Resource<Result<Vec<Recipe>, ServerFnError>> {
    use_resource(move || async move {
        let recipes: Result<Vec<Recipe>, ServerFnError> = get_recipes().await;
        recipes
    })
}

pub fn use_add_recipe() -> AddRecipe {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddRecipe { loading, result }
}

impl AddRecipe {
    pub fn submit(mut self, new_recipe: NewRecipe) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let r = add_recipe(new_recipe).await;
            self.result.set(Some(r));
            self.loading.set(false);
        });
    }
}
