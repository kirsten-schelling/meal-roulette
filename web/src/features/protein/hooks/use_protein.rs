use crate::server::api::protein::{add_protein, get_proteins};
use dioxus::prelude::*;
use domain::protein::{NewProtein, Protein};

#[derive(Clone, Copy)]
pub struct AddProtein {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Protein, ServerFnError>>>,
}

pub fn use_get_proteins() -> Resource<Result<Vec<Protein>, ServerFnError>> {
    use_resource(move || async move {
        let proteins: Result<Vec<Protein>, ServerFnError> = get_proteins().await;
        proteins
    })
}

pub fn use_add_protein() -> AddProtein {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddProtein { loading, result }
}

impl AddProtein {
    pub fn submit(mut self, new_protein: NewProtein) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let p = add_protein(new_protein).await;
            self.result.set(Some(p));
            self.loading.set(false);
        });
    }
}
