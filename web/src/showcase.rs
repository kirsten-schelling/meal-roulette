//! Temporary component gallery for the locked-in "Warm" theme. Toggle dark mode
//! and preview the accent (green vs purple). Delete once we rebuild the real
//! pages on these primitives.

use crate::components::ui::*;
use dioxus::prelude::*;

#[component]
pub fn Showcase() -> Element {
    let mut dark = use_signal(|| false);
    let mut accent = use_signal(|| "green".to_string());
    let mut tab = use_signal(|| 0usize);
    let mut modal_open = use_signal(|| false);

    // form state
    let mut title = use_signal(|| "Pad Thai".to_string());
    let mut difficulty = use_signal(|| "3".to_string());
    let mut cuisine = use_signal(|| "thai".to_string());
    let mut vegetarian = use_signal(|| true);
    let mut notify = use_signal(|| false);

    let theme = if dark() { "dark" } else { "light" };
    let acc = accent();
    let green_variant = if acc == "green" {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };
    let purple_variant = if acc == "purple" {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let tab_body = match tab() {
        0 => "Monday: Pad Thai · Tuesday: Tacos · Wednesday: Ramen.",
        1 => "Pantry staples you already have: soy sauce, garlic, rice, eggs.",
        _ => "Aubergine, coconut milk, lime, peanuts, rice noodles.",
    };

    rsx! {
        div {
            class: "mr-scope mr-gallery",
            "data-theme": "{theme}",
            "data-accent": "{acc}",

            div { class: "mr-gallery__inner",

                // ── toolbar ────────────────────────────────────────────────
                div { class: "mr-gallery__bar",
                    div {
                        Heading { level: 1, "Warm — component gallery" }
                        Text { muted: true, "The locked-in vibe. Flip dark mode and try the accent." }
                    }
                    HStack { gap: 1.25, align: Align::Center,
                        HStack { gap: 0.4, align: Align::Center,
                            Button {
                                variant: green_variant,
                                size: Size::Sm,
                                onclick: move |_| accent.set("green".to_string()),
                                "Green"
                            }
                            Button {
                                variant: purple_variant,
                                size: Size::Sm,
                                onclick: move |_| accent.set("purple".to_string()),
                                "Purple"
                            }
                        }
                        Switch { checked: dark(), onchange: move |v| dark.set(v), label: "Dark mode".to_string() }
                    }
                }

                // ── buttons ────────────────────────────────────────────────
                div { class: "mr-section",
                    Card { header: "Buttons".to_string(),
                        VStack { gap: 0.7,
                            HStack { gap: 0.5, align: Align::Center,
                                Button { variant: ButtonVariant::Solid, size: Size::Sm, "Small" }
                                Button { variant: ButtonVariant::Solid, "Medium" }
                                Button { variant: ButtonVariant::Solid, size: Size::Lg, "Large" }
                            }
                            HStack { gap: 0.5, align: Align::Center,
                                Button { variant: ButtonVariant::Outline, "Outline" }
                                Button { variant: ButtonVariant::Ghost, "Ghost" }
                                Button { variant: ButtonVariant::Danger, "Delete" }
                                Button { variant: ButtonVariant::Solid, loading: true, "Saving" }
                            }
                        }
                    }
                }

                // ── form controls ──────────────────────────────────────────
                div { class: "mr-section",
                    Card { header: "Form controls".to_string(),
                        VStack { gap: 0.9,
                            Input {
                                label: "Title".to_string(),
                                value: title(),
                                oninput: move |e: FormEvent| title.set(e.value()),
                                placeholder: "e.g. Pad Thai".to_string(),
                            }
                            Select {
                                label: "Difficulty".to_string(),
                                value: difficulty(),
                                onchange: move |v: String| difficulty.set(v),
                                options: vec![
                                    SelectOption::new("1", "Very easy"),
                                    SelectOption::new("2", "Easy"),
                                    SelectOption::new("3", "Medium"),
                                    SelectOption::new("4", "Hard"),
                                    SelectOption::new("5", "Ambitious"),
                                ],
                            }
                            Select {
                                label: "Cuisine (searchable)".to_string(),
                                value: cuisine(),
                                onchange: move |v: String| cuisine.set(v),
                                searchable: true,
                                options: vec![
                                    SelectOption::new("thai", "Thai"),
                                    SelectOption::new("italian", "Italian"),
                                    SelectOption::new("mexican", "Mexican"),
                                    SelectOption::new("japanese", "Japanese"),
                                    SelectOption::new("indian", "Indian"),
                                    SelectOption::new("chinese", "Chinese"),
                                    SelectOption::new("french", "French"),
                                    SelectOption::new("greek", "Greek"),
                                    SelectOption::new("spanish", "Spanish"),
                                    SelectOption::new("korean", "Korean"),
                                    SelectOption::new("vietnamese", "Vietnamese"),
                                    SelectOption::new("turkish", "Turkish"),
                                    SelectOption::new("lebanese", "Lebanese"),
                                    SelectOption::new("moroccan", "Moroccan"),
                                    SelectOption::new("ethiopian", "Ethiopian"),
                                    SelectOption::new("brazilian", "Brazilian"),
                                ],
                            }
                            HStack { gap: 1.5, align: Align::Center,
                                Checkbox {
                                    checked: vegetarian(),
                                    onchange: move |v| vegetarian.set(v),
                                    label: "Vegetarian".to_string(),
                                }
                                Switch {
                                    checked: notify(),
                                    onchange: move |v| notify.set(v),
                                    label: "Notify me".to_string(),
                                }
                            }
                        }
                    }
                }

                // ── status: badges, alerts, loaders ────────────────────────
                div { class: "mr-section",
                    Card { header: "Status & feedback".to_string(),
                        VStack { gap: 0.9,
                            HStack { gap: 0.5, align: Align::Center,
                                Badge { tone: Tone::Green, "Easy" }
                                Badge { tone: Tone::Amber, "Medium" }
                                Badge { tone: Tone::Red, "Ambitious" }
                                Badge { tone: Tone::Neutral, "Draft" }
                            }
                            Alert { tone: Tone::Green, title: "Recipe added".to_string(),
                                "“Pad Thai” is now in your library."
                            }
                            Alert { tone: Tone::Amber, "Heads up — 3 ingredients are missing from your pantry." }
                            HStack { gap: 1.0, align: Align::Center,
                                Spinner { size: Size::Sm }
                                Spinner { size: Size::Md }
                                Spinner { size: Size::Lg }
                            }
                            VStack { gap: 0.5,
                                Skeleton { width: "70%".to_string(), height: "1rem".to_string() }
                                Skeleton { width: "100%".to_string(), height: "1rem".to_string() }
                                Skeleton { width: "45%".to_string(), height: "1rem".to_string() }
                            }
                        }
                    }
                }

                // ── table ──────────────────────────────────────────────────
                div { class: "mr-section",
                    Card { header: "This week".to_string(),
                        Table {
                            thead {
                                tr {
                                    th { "Title" }
                                    th { "Difficulty" }
                                    th { "Time" }
                                }
                            }
                            tbody {
                                tr {
                                    td { "Pad Thai" }
                                    td { Badge { tone: Tone::Green, "Easy" } }
                                    td { "25 min" }
                                }
                                tr {
                                    td { "Tacos" }
                                    td { Badge { tone: Tone::Amber, "Medium" } }
                                    td { "40 min" }
                                }
                                tr {
                                    td { "Beef Wellington" }
                                    td { Badge { tone: Tone::Red, "Ambitious" } }
                                    td { "120 min" }
                                }
                            }
                        }
                    }
                }

                // ── tabs ───────────────────────────────────────────────────
                div { class: "mr-section",
                    Card { header: "Tabs".to_string(),
                        VStack { gap: 0.8,
                            Tabs {
                                labels: vec!["Plan".to_string(), "Pantry".to_string(), "Shopping".to_string()],
                                selected: tab(),
                                on_select: move |i| tab.set(i),
                            }
                            Text { "{tab_body}" }
                        }
                    }
                }

                // ── typography + separator + modal ─────────────────────────
                div { class: "mr-section",
                    Card { header: "Text & overlays".to_string(),
                        VStack { gap: 0.7,
                            Heading { level: 2, "A serif heading" }
                            Text { "Body text sits in the sans-serif face for easy reading." }
                            Text { muted: true, small: true, "Muted small text — captions, hints, metadata." }
                            Separator { label: "then".to_string() }
                            Button {
                                variant: ButtonVariant::Solid,
                                onclick: move |_| modal_open.set(true),
                                "Open dialog"
                            }
                        }
                    }
                }
            }

            Modal {
                open: modal_open(),
                on_close: move |_| modal_open.set(false),
                title: "Delete recipe?".to_string(),
                VStack { gap: 1.0,
                    Text { "This removes “Pad Thai” from your library. You can't undo this." }
                    HStack { gap: 0.6, align: Align::Center,
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| modal_open.set(false),
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: move |_| modal_open.set(false),
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
