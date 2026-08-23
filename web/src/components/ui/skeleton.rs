use dioxus::prelude::*;

/// A shimmering placeholder block for loading states. `width`/`height` accept
/// any CSS length (default full width, 1rem tall).
#[component]
pub fn Skeleton(
    #[props(default = "100%".to_string())] width: String,
    #[props(default = "1rem".to_string())] height: String,
    #[props(optional)] radius: Option<String>,
) -> Element {
    let r = radius
        .clone()
        .unwrap_or_else(|| "var(--mr-radius-sm)".to_string());
    let style = format!("width:{width};height:{height};border-radius:{r};");
    rsx! {
        div { class: "mr-skeleton", style: "{style}" }
    }
}
