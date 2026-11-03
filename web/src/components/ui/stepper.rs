use dioxus::prelude::*;

/// A whole-number field with − and + buttons. The typed text is kept as is
/// while editing, so clearing the field to type a new number does not snap it
/// back; the value is clamped to `min..=max` when the field loses focus.
/// `label` names the field for assistive tech, since the stepper sits inside a
/// sentence rather than under a visible label.
#[component]
pub fn NumberStepper(
    value: ReadSignal<u32>,
    onchange: EventHandler<u32>,
    label: String,
    #[props(default = 0)] min: u32,
    #[props(default = 99)] max: u32,
) -> Element {
    let mut text = use_signal(|| value().to_string());
    use_effect(move || text.set(value().to_string()));

    let current = move || {
        text()
            .trim()
            .parse::<u32>()
            .unwrap_or_else(|_| value())
            .clamp(min, max)
    };
    let mut set = move |n: u32| {
        text.set(n.to_string());
        onchange.call(n);
    };
    let now = current();

    rsx! {
        span { class: "mr-stepper", role: "group", aria_label: "{label}",
            button {
                r#type: "button",
                class: "mr-stepper__btn",
                aria_label: "Fewer",
                disabled: now <= min,
                onclick: move |_| set(current().saturating_sub(1).max(min)),
                "−"
            }
            input {
                class: "mr-stepper__input",
                r#type: "text",
                inputmode: "numeric",
                role: "spinbutton",
                aria_label: "{label}",
                aria_valuemin: "{min}",
                aria_valuemax: "{max}",
                aria_valuenow: "{value()}",
                value: "{text}",
                oninput: move |e| {
                    let typed = e.value();
                    let parsed = typed.trim().parse::<u32>();
                    text.set(typed);
                    if let Ok(n) = parsed {
                        if (min..=max).contains(&n) {
                            onchange.call(n);
                        }
                    }
                },
                onblur: move |_| {
                    let n = text().trim().parse::<u32>().unwrap_or(min).clamp(min, max);
                    set(n);
                },
                onkeydown: move |e: KeyboardEvent| match e.key() {
                    Key::ArrowUp => {
                        e.prevent_default();
                        set((current() + 1).min(max));
                    }
                    Key::ArrowDown => {
                        e.prevent_default();
                        set(current().saturating_sub(1).max(min));
                    }
                    _ => {}
                },
            }
            button {
                r#type: "button",
                class: "mr-stepper__btn",
                aria_label: "More",
                disabled: now >= max,
                onclick: move |_| set((current() + 1).min(max)),
                "+"
            }
        }
    }
}
