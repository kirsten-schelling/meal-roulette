use crate::server::api::recipe::{
    add_recipe, get_recipe_detail, get_recipe_overviews, get_recipes,
};
use dioxus::prelude::*;
use domain::recipe::{NewRecipe, Recipe, RecipeDetail, RecipeOverview};
use uuid::Uuid;

pub use crate::server::api::recipe::{
    add_recipe_ingredient, copy_recipe, delete_recipe, remove_recipe_ingredient, update_recipe,
    update_recipe_ingredient,
};

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

pub fn use_get_recipe_overviews() -> Resource<Result<Vec<RecipeOverview>, ServerFnError>> {
    use_resource(move || async move { get_recipe_overviews().await })
}

/// Refetches when the route moves to another recipe, not only on first mount.
pub fn use_get_recipe_detail(id: Uuid) -> Resource<Result<RecipeDetail, ServerFnError>> {
    use_resource(use_reactive(&id, |id| async move {
        get_recipe_detail(id).await
    }))
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
