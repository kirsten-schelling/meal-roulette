use crate::server::api::unit::{add_unit, delete_unit, get_units, update_unit};
use dioxus::prelude::*;
use domain::unit::{NewUnit, Unit, UnitWithIngredientList, UpdateUnit};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct AddUnit {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Unit, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct EditUnit {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Unit, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct DeleteUnit {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Unit, ServerFnError>>>,
}

pub fn use_get_units() -> Resource<Result<Vec<UnitWithIngredientList>, ServerFnError>> {
    use_resource(move || async move {
        let units: Result<Vec<UnitWithIngredientList>, ServerFnError> = get_units().await;
        units
    })
}

pub fn use_add_unit() -> AddUnit {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddUnit { loading, result }
}

pub fn use_edit_unit() -> EditUnit {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    EditUnit { loading, result }
}

pub fn use_delete_unit() -> DeleteUnit {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    DeleteUnit { loading, result }
}

impl AddUnit {
    pub fn submit(mut self, new_unit: NewUnit) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let u = add_unit(new_unit).await;
            self.result.set(Some(u));
            self.loading.set(false);
        });
    }
}

impl EditUnit {
    pub fn submit(mut self, id: Uuid, update: UpdateUnit) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let u = update_unit(id, update).await;
            self.result.set(Some(u));
            self.loading.set(false);
        });
    }
}

impl DeleteUnit {
    pub fn submit(mut self, id: Uuid) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let u = delete_unit(id).await;
            self.result.set(Some(u));
            self.loading.set(false);
        });
    }
}
