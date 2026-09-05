use crate::server::api::unit::{add_unit, get_units};
use dioxus::prelude::*;
use domain::unit::{NewUnit, Unit, UnitWithIngredientList};

#[derive(Clone, Copy)]
pub struct AddUnit {
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
