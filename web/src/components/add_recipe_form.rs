use crate::server::add_recipe;
use dioxus::prelude::*;

/// Owns its own form state; calls the `add_recipe` server function, then fires
/// `on_added` so the parent can re-fetch.
#[component]
pub fn AddRecipeForm(on_added: EventHandler<()>) -> Element {
    let mut title = use_signal(String::new);
    let mut difficulty = use_signal(|| 3i32);
    let mut time_mins = use_signal(|| 30i32);

    let submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        let t = title();
        if t.trim().is_empty() {
            return;
        }
        let _ = add_recipe(t, difficulty(), time_mins()).await;
        title.set(String::new());
        on_added.call(());
    };

    rsx! {
        article {
            h3 { "Add a recipe" }
            form { onsubmit: submit,
                div { class: "grid",
                    label {
                        "Title"
                        input {
                            r#type: "text",
                            placeholder: "e.g. Pad Thai",
                            value: "{title}",
                            oninput: move |e| title.set(e.value()),
                        }
                    }
                    label {
                        "Difficulty"
                        input {
                            r#type: "number",
                            min: "1",
                            max: "5",
                            value: "{difficulty}",
                            oninput: move |e| difficulty.set(e.value().parse().unwrap_or(3)),
                        }
                    }
                    label {
                        "Time (min)"
                        input {
                            r#type: "number",
                            min: "0",
                            value: "{time_mins}",
                            oninput: move |e| time_mins.set(e.value().parse().unwrap_or(30)),
                        }
                    }
                }
                button { r#type: "submit", "Add recipe" }
            }
        }
    }
}
