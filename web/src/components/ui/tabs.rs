use dioxus::prelude::*;

/// A row of tab buttons. Controlled: `selected` is the active index; clicking a
/// tab fires `on_select` with its index. The parent decides what content to show
/// for the selected tab.
///
/// Keyboard: only the selected tab is a tab stop; Left/Right (and Home/End) move to
/// another tab and select it, wrapping at the ends. Give the content area
/// `role="tabpanel"` and pass its id as `panel_id` to link the two.
#[component]
pub fn Tabs(
    labels: Vec<String>,
    selected: usize,
    on_select: EventHandler<usize>,
    #[props(optional)] panel_id: Option<String>,
) -> Element {
    let base = use_hook(|| format!("mr-tabs-{}", dioxus::core::current_scope_id().0));
    let count = labels.len();

    rsx! {
        div { class: "mr-tabs", role: "tablist",
            for (i , label) in labels.iter().enumerate() {
                button {
                    key: "{i}",
                    id: "{base}-{i}",
                    r#type: "button",
                    class: if i == selected { "mr-tab mr-tab--active" } else { "mr-tab" },
                    role: "tab",
                    aria_selected: i == selected,
                    aria_controls: panel_id.clone(),
                    tabindex: if i == selected { "0" } else { "-1" },
                    onclick: move |_| on_select.call(i),
                    onkeydown: {
                        let base = base.clone();
                        move |e: KeyboardEvent| {
                            let target = match e.key() {
                                Key::ArrowRight => (i + 1) % count,
                                Key::ArrowLeft => (i + count - 1) % count,
                                Key::Home => 0,
                                Key::End => count - 1,
                                _ => return,
                            };
                            e.prevent_default();
                            on_select.call(target);
                            let _ = document::eval(
                                &format!("document.getElementById('{base}-{target}')?.focus();"),
                            );
                        }
                    },
                    "{label}"
                }
            }
        }
    }
}
