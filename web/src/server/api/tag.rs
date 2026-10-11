use dioxus::prelude::*;
use domain::tag::{NewTag, Tag, TagWithRecipeList, UpdateTag};
use uuid::Uuid;

#[server]
pub async fn get_tags() -> Result<Vec<TagWithRecipeList>, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::tag::repository;

    let state = app_state()?;
    repository::list(&state.db).await.map_err(server_fn_error)
}

#[server]
pub async fn add_tag(tag: NewTag) -> Result<Tag, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::tag::repository;

    let state = app_state()?;
    repository::create(&state.db, tag)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn update_tag(id: Uuid, tag: UpdateTag) -> Result<Tag, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::tag::repository;

    let state = app_state()?;
    repository::update(&state.db, id, tag)
        .await
        .map_err(server_fn_error)
}

#[server]
pub async fn delete_tag(id: Uuid) -> Result<Tag, ServerFnError> {
    use crate::server::{app_state, server_fn_error};
    use api::features::tag::repository;

    let state = app_state()?;
    repository::delete(&state.db, id)
        .await
        .map_err(server_fn_error)
}
