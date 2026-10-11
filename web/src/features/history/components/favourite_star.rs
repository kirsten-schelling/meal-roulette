use dioxus::prelude::*;

/// A star that toggles a plan's favourite flag. `on` is the current state;
/// `onclick` fires when clicked, and is ignored while `busy`.
#[component]
pub fn FavouriteStar(
    on: bool,
    #[props(default)] busy: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class = if on {
        "mr-iconbtn mr-star mr-star--on"
    } else {
        "mr-iconbtn mr-star"
    };
    let label = if on {
        "Remove from favourites"
    } else {
        "Add to favourites"
    };
    rsx! {
        button {
            r#type: "button",
            class,
            title: label,
            aria_label: label,
            aria_pressed: on,
            disabled: busy,
            onclick: move |e| onclick.call(e),
            if on {
                "★"
            } else {
                "☆"
            }
        }
    }
}
