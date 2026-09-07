use dioxus::prelude::*;
use domain::protein::{NewProtein, Protein, ProteinWithRecipeList};

#[server]
pub async fn get_proteins() -> Result<Vec<ProteinWithRecipeList>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::protein::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_protein(protein: NewProtein) -> Result<Protein, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::protein::repository;

    let state = app_state()?;
    repository::create(&state.db, protein)
        .await
        .map_err(server_fn_error)
}
