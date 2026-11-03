use crate::server::api::protein::{add_protein, delete_protein, get_proteins, update_protein};
pub use crate::server::api::protein::{delete_protein_cascade, get_protein_references};
use dioxus::prelude::*;
use domain::protein::{NewProtein, Protein, ProteinWithRecipeList, UpdateProtein};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct AddProtein {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Protein, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct EditProtein {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Protein, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct DeleteProtein {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Protein, ServerFnError>>>,
}

pub fn use_get_proteins() -> Resource<Result<Vec<ProteinWithRecipeList>, ServerFnError>> {
    use_resource(move || async move {
        let proteins: Result<Vec<ProteinWithRecipeList>, ServerFnError> = get_proteins().await;
        proteins
    })
}

pub fn use_add_protein() -> AddProtein {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddProtein { loading, result }
}

pub fn use_edit_protein() -> EditProtein {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    EditProtein { loading, result }
}

pub fn use_delete_protein() -> DeleteProtein {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    DeleteProtein { loading, result }
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

impl EditProtein {
    pub fn submit(mut self, id: Uuid, update: UpdateProtein) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let p = update_protein(id, update).await;
            self.result.set(Some(p));
            self.loading.set(false);
        });
    }
}

impl DeleteProtein {
    pub fn submit(mut self, id: Uuid) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let p = delete_protein(id).await;
            self.result.set(Some(p));
            self.loading.set(false);
        });
    }
}
