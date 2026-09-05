use dioxus::prelude::*;
use domain::protein::{NewProtein, Protein, ProteinWithRecipeList};

#[server]
pub async fn get_proteins() -> Result<Vec<ProteinWithRecipeList>, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let proteins: Vec<ProteinWithRecipeList> = send_request(|c| c.get(get_url("proteins")))
        .await
        .map_err(|e| ServerFnError::ServerError {
        message: e.to_string(),
        code: 500,
        details: None,
    })?;
    Ok(proteins)
}

#[server]
pub async fn add_protein(protein: NewProtein) -> Result<Protein, ServerFnError> {
    use crate::server::fetcher::{get_url, send_request};
    let protein: Protein = send_request(|c| {
        c.post(get_url("proteins"))
            .body(serde_json::to_string(&protein).unwrap())
    })
    .await
    .map_err(|e| ServerFnError::ServerError {
        message: e.to_string(),
        code: 500,
        details: None,
    })?;
    Ok(protein)
}
