use crate::server::api::tag::{add_tag, delete_tag, get_tags, update_tag};
use dioxus::prelude::*;
use domain::tag::{NewTag, Tag, TagWithRecipeList, UpdateTag};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct AddTag {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Tag, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct EditTag {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Tag, ServerFnError>>>,
}

#[derive(Clone, Copy)]
pub struct DeleteTag {
    pub loading: Signal<bool>,
    pub result: Signal<Option<Result<Tag, ServerFnError>>>,
}

pub fn use_get_tags() -> Resource<Result<Vec<TagWithRecipeList>, ServerFnError>> {
    use_resource(move || async move {
        let tags: Result<Vec<TagWithRecipeList>, ServerFnError> = get_tags().await;
        tags
    })
}

pub fn use_add_tag() -> AddTag {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    AddTag { loading, result }
}

pub fn use_edit_tag() -> EditTag {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    EditTag { loading, result }
}

pub fn use_delete_tag() -> DeleteTag {
    let loading = use_signal(|| false);
    let result = use_signal(|| None);
    DeleteTag { loading, result }
}

impl AddTag {
    pub fn submit(mut self, new_tag: NewTag) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let c = add_tag(new_tag).await;
            self.result.set(Some(c));
            self.loading.set(false);
        });
    }
}

impl EditTag {
    pub fn submit(mut self, id: Uuid, update: UpdateTag) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let c = update_tag(id, update).await;
            self.result.set(Some(c));
            self.loading.set(false);
        });
    }
}

impl DeleteTag {
    pub fn submit(mut self, id: Uuid) {
        if (self.loading)() {
            return;
        }
        spawn(async move {
            self.loading.set(true);
            let c = delete_tag(id).await;
            self.result.set(Some(c));
            self.loading.set(false);
        });
    }
}
