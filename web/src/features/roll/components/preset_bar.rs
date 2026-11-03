use crate::components::ui::*;
use crate::features::roll::hooks::use_roll_preset::{
    add_roll_preset, delete_roll_preset, set_default_roll_preset, update_roll_preset,
    use_get_roll_presets,
};
use crate::features::roll::settings::{RollSettings, MAX_ROLL_DAYS};
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use dioxus::prelude::*;
use domain::roll_preset::{NewRollPreset, RollPreset, UpdateRollPreset};
use uuid::Uuid;

/// A preset's settings as the page holds them: an older preset's difficulty mix
/// turned into rules, and a day count that is a number in range.
fn read_settings(preset: &RollPreset) -> Option<RollSettings> {
    let mut settings = serde_json::from_value::<RollSettings>(preset.settings.clone())
        .ok()?
        .normalized();
    settings.num_days = settings.days().clamp(1, MAX_ROLL_DAYS).to_string();
    Some(settings)
}

/// What the confirm dialog is asking about.
#[derive(Clone, PartialEq)]
enum Ask {
    Update(Uuid, String),
    Delete(Uuid, String),
}

/// The preset chip: picking a preset applies it at once. The ⋯ menu saves the
/// current rules as a new preset, updates or deletes the chosen one, or makes
/// it the default. "(edited)" shows when the rules no longer match it.
#[component]
pub fn PresetBar(settings: Signal<RollSettings>) -> Element {
    let presets = use_get_roll_presets();
    let toast = use_toast();
    let mut selected = use_signal(|| None as Option<Uuid>);
    // What the chosen preset holds, to tell whether the rules have been edited.
    let mut baseline = use_signal(|| None as Option<RollSettings>);
    let mut applied_default = use_signal(|| false);
    let mut ask = use_signal(|| None as Option<Ask>);
    let mut naming = use_signal(|| None as Option<String>);
    let mut busy = use_signal(|| false);

    let mut apply = move |preset: &RollPreset| match read_settings(preset) {
        Some(read) => {
            baseline.set(Some(read.clone()));
            settings.set(read);
            selected.set(Some(preset.id));
        }
        None => toast.show(
            Some("Couldn't load".to_string()),
            "This preset's settings couldn't be read.".to_string(),
            Tone::Red,
        ),
    };

    use_effect(move || {
        if applied_default() {
            return;
        }
        if let Some(Ok(list)) = presets() {
            if let Some(preset) = list.iter().find(|p| p.is_default) {
                apply(preset);
            }
            applied_default.set(true);
        }
    });

    let list = match presets() {
        Some(Ok(list)) => list,
        Some(Err(e)) => {
            return rsx! {
                Text { small: true, muted: true, "Presets unavailable: {error_message(&e)}" }
            };
        }
        None => return rsx! {},
    };
    let chosen = selected().and_then(|id| list.iter().find(|p| p.id == id).cloned());
    let edited = chosen.is_some() && baseline().as_ref() != Some(&settings());
    let mut options = vec![SelectOption::new("", "No preset")];
    options.extend(list.iter().map(|p| {
        let label = if p.is_default {
            format!("{} · default", p.name)
        } else {
            p.name.clone()
        };
        SelectOption::new(p.id.to_string(), label)
    }));
    let value = chosen
        .as_ref()
        .map(|p| p.id.to_string())
        .unwrap_or_default();
    let chosen_name = chosen.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let can_make_default = chosen.as_ref().is_some_and(|p| !p.is_default);
    let update_label = if chosen_name.is_empty() {
        "Update preset".to_string()
    } else {
        format!("Update “{chosen_name}”")
    };

    let on_pick = {
        let list = list.clone();
        move |picked: String| {
            match list.iter().find(|p| p.id.to_string() == picked) {
                Some(preset) => apply(preset),
                None => {
                    selected.set(None);
                    baseline.set(None);
                }
            };
        }
    };

    let mut update = move |id: Uuid| {
        let current = settings();
        let Ok(payload) = serde_json::to_value(&current) else {
            return;
        };
        let mut presets = presets;
        busy.set(true);
        spawn(async move {
            match update_roll_preset(
                id,
                UpdateRollPreset {
                    name: None,
                    settings: Some(payload),
                },
            )
            .await
            {
                Ok(_) => {
                    baseline.set(Some(current));
                    presets.restart();
                    toast.show(
                        Some("Updated".to_string()),
                        "Preset updated.".to_string(),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Update failed".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            busy.set(false);
            ask.set(None);
        });
    };

    let mut delete = move |id: Uuid| {
        let mut presets = presets;
        busy.set(true);
        spawn(async move {
            match delete_roll_preset(id).await {
                Ok(_) => {
                    selected.set(None);
                    baseline.set(None);
                    presets.restart();
                    toast.show(
                        Some("Deleted".to_string()),
                        "Preset removed.".to_string(),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Delete failed".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            busy.set(false);
            ask.set(None);
        });
    };

    let make_default = move |id: Uuid| {
        let mut presets = presets;
        spawn(async move {
            match set_default_roll_preset(id).await {
                Ok(_) => {
                    presets.restart();
                    toast.show(
                        Some("Default set".to_string()),
                        "This preset now loads when you open Roll.".to_string(),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't set default".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
        });
    };

    let mut save_new = move |name: String| {
        let current = settings();
        let Ok(payload) = serde_json::to_value(&current) else {
            return;
        };
        let mut presets = presets;
        busy.set(true);
        spawn(async move {
            match add_roll_preset(NewRollPreset {
                household_id: None,
                name,
                settings: payload,
            })
            .await
            {
                Ok(saved) => {
                    selected.set(Some(saved.id));
                    baseline.set(Some(current));
                    presets.restart();
                    naming.set(None);
                    toast.show(
                        Some("Saved".to_string()),
                        "Preset saved.".to_string(),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Save failed".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            busy.set(false);
        });
    };

    let chosen_id = chosen.as_ref().map(|p| p.id);

    rsx! {
        div { class: "mr-presetbar",
            if let Some(Ask::Update(id, name)) = ask() {
                ConfirmDialog {
                    title: format!("Update the preset “{name}”?"),
                    confirm_label: "Update".to_string(),
                    loading: busy(),
                    on_confirm: move |_| update(id),
                    on_cancel: move |_| ask.set(None),
                    Text { "It will hold the rules as they are now." }
                }
            }
            if let Some(Ask::Delete(id, name)) = ask() {
                ConfirmDialog {
                    title: format!("Delete the preset “{name}”?"),
                    confirm_label: "Delete".to_string(),
                    danger: true,
                    loading: busy(),
                    on_confirm: move |_| delete(id),
                    on_cancel: move |_| ask.set(None),
                    Text { "Your saved rolls are not affected." }
                }
            }
            if let Some(name) = naming() {
                Modal { title: "Save as new preset".to_string(), on_close: move |_| naming.set(None),
                    Input {
                        label: "Name".to_string(),
                        value: name.clone(),
                        placeholder: "Weeknights".to_string(),
                        autofocus: true,
                        oninput: move |e: FormEvent| naming.set(Some(e.value())),
                    }
                    div { class: "mr-dialog-foot",
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| naming.set(None),
                            "Cancel"
                        }
                        Button {
                            loading: busy(),
                            disabled: name.trim().is_empty(),
                            onclick: {
                                let name = name.trim().to_string();
                                move |_| save_new(name.clone())
                            },
                            "Save"
                        }
                    }
                }
            }
            span { class: "mr-presetbar__label", "Preset" }
            Select {
                compact: true,
                aria_label: "Preset".to_string(),
                value,
                options,
                onchange: on_pick,
            }
            if edited {
                span { class: "mr-presetbar__edited", "(edited)" }
            }
            ActionMenu { label: "More preset actions".to_string(),
                ActionMenuItem {
                    onclick: move |_| naming.set(Some(String::new())),
                    "Save as new preset…"
                }
                ActionMenuItem {
                    disabled: chosen_id.is_none() || !edited,
                    onclick: {
                        let name = chosen_name.clone();
                        move |_| {
                            if let Some(id) = chosen_id {
                                ask.set(Some(Ask::Update(id, name.clone())));
                            }
                        }
                    },
                    "{update_label}"
                }
                ActionMenuItem {
                    disabled: !can_make_default,
                    onclick: move |_| {
                        if let Some(id) = chosen_id {
                            make_default(id);
                        }
                    },
                    "Make default"
                }
                ActionMenuItem {
                    danger: true,
                    disabled: chosen_id.is_none(),
                    onclick: {
                        let name = chosen_name.clone();
                        move |_| {
                            if let Some(id) = chosen_id {
                                ask.set(Some(Ask::Delete(id, name.clone())));
                            }
                        }
                    },
                    "Delete preset"
                }
            }
        }
    }
}
