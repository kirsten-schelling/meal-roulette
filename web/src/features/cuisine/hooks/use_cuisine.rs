use crate::server::api::cuisine::{add_cuisine, delete_cuisine, get_cuisines, update_cuisine};
pub use crate::server::api::cuisine::{delete_cuisine_cascade, get_cuisine_references};
use dioxus::prelude::*;
use domain::cuisine::{Cuisine, CuisineWithRecipeList, NewCuisine, UpdateCuisine};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct AddCuisine {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Cuisine, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct EditCuisine {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Cuisine, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct DeleteCuisine {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Cuisine, ServerFnError>>>,
}

pub fn use_get_cuisines() -> Resource<Result<Vec<CuisineWithRecipeList>, ServerFnError>> {
    use_resource(move || async move {
        let cuisines: Result<Vec<CuisineWithRecipeList>, ServerFnError> = get_cuisines().await;
        cuisines
    })
}

pub fn use_add_cuisine() -> AddCuisine {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddCuisine { loading, result }
}

pub fn use_edit_cuisine() -> EditCuisine {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    EditCuisine { loading, result }
}

pub fn use_delete_cuisine() -> DeleteCuisine {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    DeleteCuisine { loading, result }
}

impl AddCuisine {
    pub fn submit(mut self, new_cuisine: NewCuisine) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let c = add_cuisine(new_cuisine).await;
            self.result.set(Some(c));
            self.loading.set(false);
        });
    }
}

impl EditCuisine {
    pub fn submit(mut self, id: Uuid, update: UpdateCuisine) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let c = update_cuisine(id, update).await;
            self.result.set(Some(c));
            self.loading.set(false);
        });
    }
}

impl DeleteCuisine {
    pub fn submit(mut self, id: Uuid) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let c = delete_cuisine(id).await;
            self.result.set(Some(c));
            self.loading.set(false);
        });
    }
}
