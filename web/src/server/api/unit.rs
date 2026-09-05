use dioxus::prelude::*;
use domain::unit::{NewUnit, Unit, UnitWithIngredientList};

#[server]
pub async fn get_units() -> Result<Vec<UnitWithIngredientList>, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let units: Vec<UnitWithIngredientList> = send_request(|c| c.get(get_url("units")))
        .await
        .map_err(|e| ServerFnError::ServerError {
            message: e.to_string(),
            code: 500,
            details: None,
        })?;
    Ok(units)
}

#[server]
pub async fn add_unit(unit: NewUnit) -> Result<Unit, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let unit: Unit = send_request(|c| {
        c.post(get_url("units"))
            .body(serde_json::to_string(&unit).unwrap())
    })
    .await
    .map_err(|e| ServerFnError::ServerError {
        message: e.to_string(),
        code: 500,
        details: None,
    })?;
    Ok(unit)
}
