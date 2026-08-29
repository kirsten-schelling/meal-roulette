use crate::server::api::ingredient::{add_ingredient, get_ingredients};
use dioxus::prelude::*;
use domain::ingredient::{Ingredient, NewIngredient};

#[derive(Clone, Copy)]
pub struct AddIngredient {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Ingredient, ServerFnError>>>,
}

pub fn use_get_ingredients() -> Resource<Result<Vec<Ingredient>, ServerFnError>> {
    use_resource(move || async move {
        let ingredients: Result<Vec<Ingredient>, ServerFnError> = get_ingredients().await;
        ingredients
    })
}

pub fn use_add_ingredient() -> AddIngredient {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddIngredient { loading, result }
}

impl AddIngredient {
    pub fn submit(mut self, new_ingredient: NewIngredient) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let i = add_ingredient(new_ingredient).await;
            self.result.set(Some(i));
            self.loading.set(false);
        });
    }
}
