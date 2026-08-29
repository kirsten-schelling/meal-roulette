use crate::server::api::cuisine::{add_cuisine, get_cuisines};
use dioxus::prelude::*;
use domain::cuisine::{Cuisine, NewCuisine};

#[derive(Clone, Copy)]
pub struct AddCuisine {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Cuisine, ServerFnError>>>,
}

pub fn use_get_cuisines() -> Resource<Result<Vec<Cuisine>, ServerFnError>> {
    use_resource(move || async move {
        let cuisines: Result<Vec<Cuisine>, ServerFnError> = get_cuisines().await;
        cuisines
    })
}

pub fn use_add_cuisine() -> AddCuisine {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddCuisine { loading, result }
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
