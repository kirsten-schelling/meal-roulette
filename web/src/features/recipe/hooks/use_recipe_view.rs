//! Whether the recipes page shows cards or the table. The choice lives in
//! `localStorage`; it is read once the app runs in the browser so the server
//! render and the first client render agree.

use dioxus::prelude::*;

const STORAGE_KEY: &str = "mr-recipe-view";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecipeView {
    Table,
    Cards,
}

impl RecipeView {
    fn stored(self) -> &'static str {
        match self {
            RecipeView::Table => "table",
            RecipeView::Cards => "cards",
        }
    }

    fn from_stored(value: &str) -> RecipeView {
        match value {
            "cards" => RecipeView::Cards,
            _ => RecipeView::Table,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RecipeViewChoice {
    view: Signal<RecipeView>,
}

impl RecipeViewChoice {
    pub fn get(&self) -> RecipeView {
        (self.view)()
    }

    pub fn set(&mut self, view: RecipeView) {
        self.view.set(view);
        let _ = document::eval(&format!(
            "try{{localStorage.setItem('{STORAGE_KEY}','{}')}}catch(e){{}}",
            view.stored()
        ));
    }
}

pub fn use_recipe_view() -> RecipeViewChoice {
    let mut view = use_signal(|| RecipeView::Table);

    use_effect(move || {
        spawn(async move {
            let script = format!(
                "try{{return localStorage.getItem('{STORAGE_KEY}')}}catch(e){{return null}}"
            );
            if let Ok(value) = document::eval(&script).await {
                view.set(RecipeView::from_stored(value.as_str().unwrap_or("")));
            }
        });
    });

    RecipeViewChoice { view }
}
