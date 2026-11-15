use crate::components::ui::*;
use crate::features::roll::fix::Fix;
use dioxus::prelude::*;

/// The fixes for a conflict as a row of buttons. On a phone they stack, one
/// full-width button each.
#[component]
pub fn FixButtons(fixes: Vec<Fix>, on_fix: EventHandler<Fix>) -> Element {
    rsx! {
        div { class: "mr-alert__actions mr-fixes",
            for fix in fixes {
                Button {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    onclick: {
                        let fix = fix.clone();
                        move |_| on_fix.call(fix.clone())
                    },
                    "{fix.label()}"
                }
            }
        }
    }
}
