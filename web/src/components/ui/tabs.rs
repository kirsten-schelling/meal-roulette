use dioxus::prelude::*;

/// A row of tab buttons. Controlled: `selected` is the active index; clicking a
/// tab fires `on_select` with its index. The parent decides what content to show
/// for the selected tab.
#[component]
pub fn Tabs(
    labels: Vec<String>,
    selected: usize,
    on_select: EventHandler<usize>,
) -> Element {
    rsx! {
        div { class: "mr-tabs", role: "tablist",
            for (i, label) in labels.iter().enumerate() {
                button {
                    key: "{i}",
                    class: if i == selected { "mr-tab mr-tab--active" } else { "mr-tab" },
                    role: "tab",
                    aria_selected: i == selected,
                    onclick: move |_| on_select.call(i),
                    "{label}"
                }
            }
        }
    }
}
