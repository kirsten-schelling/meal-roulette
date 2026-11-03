use crate::components::ui::{Button, ButtonVariant, Size, Text};
use dioxus::prelude::*;
use domain::slept_ingredient::MAX_SLEEP_WEEKS;

/// The bar shown once rows are picked from their More menu: how many, how
/// long to sleep them, and a way out. `on_sleep` fires with the number of
/// weeks; `on_cancel` drops the picks.
#[component]
pub fn SleepBar(
    count: usize,
    busy: bool,
    on_sleep: EventHandler<i32>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut weeks = use_signal(|| "2".to_string());
    let parsed = weeks()
        .trim()
        .parse::<i32>()
        .ok()
        .filter(|w| (1..=MAX_SLEEP_WEEKS).contains(w));
    let unit = if parsed == Some(1) { "week" } else { "weeks" };
    let submit = move |_: MouseEvent| {
        if let Some(w) = parsed {
            on_sleep.call(w);
        }
    };

    rsx! {
        div { class: "mr-bulkbar",
            Text { small: true,
                if count == 1 {
                    "1 item to sleep"
                } else {
                    "{count} items to sleep"
                }
            }
            label { class: "mr-sleepbar",
                span { "Sleep for" }
                input {
                    class: "mr-input mr-sleepbar__weeks",
                    r#type: "number",
                    min: "1",
                    max: "{MAX_SLEEP_WEEKS}",
                    value: "{weeks}",
                    oninput: move |e: FormEvent| weeks.set(e.value()),
                }
                span { "{unit}" }
            }
            Button {
                variant: ButtonVariant::Solid,
                size: Size::Sm,
                loading: busy,
                disabled: parsed.is_none(),
                onclick: submit,
                if count == 1 {
                    "Sleep item"
                } else {
                    "Sleep items"
                }
            }
            Button {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                disabled: busy,
                onclick: move |_| on_cancel.call(()),
                "Cancel"
            }
        }
    }
}
