//! The roll setup page: pick a preset (or build fresh), choose how many days
//! and the constraints, then hit Roll. Rolling sends the cuisine and protein
//! rules to the solver and lands on the planned-meals page. Presets save the
//! whole form as a JSON payload through the roll_preset API.

use crate::components::ui::*;
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::roll::hooks::use_roll::roll_week;
use crate::features::roll::hooks::use_roll_preset::{
    add_roll_preset, delete_roll_preset, set_default_roll_preset, update_roll_preset,
    use_get_roll_presets,
};
use crate::features::roll::settings::{DifficultyDays, LookupRule, RollSettings, SavedIngredient};
use crate::hooks::use_toast::use_toast;
use crate::models::{difficulty_label, parse_iso_date};
use crate::router::Route;
use dioxus::prelude::*;
use domain::roll::{CountRule, RollConstraints, RollRequest};
use domain::roll_preset::{NewRollPreset, UpdateRollPreset};
use uuid::Uuid;

/// A fresh, empty difficulty mix: one row per level (1..5), zero days each.
fn default_difficulty_mix() -> Vec<DifficultyDays> {
    (1..=5)
        .map(|level: i16| DifficultyDays { level, days: 0 })
        .collect()
}

/// Build combobox rows from a fetched list, mapping each item to (id, name).
fn lookup_options<T>(
    fetched: &Option<Result<Vec<T>, ServerFnError>>,
    to_option: impl Fn(&T) -> ComboOption,
) -> Vec<ComboOption> {
    match fetched {
        Some(Ok(list)) => list.iter().map(to_option).collect(),
        _ => Vec::new(),
    }
}

#[component]
pub fn Roll() -> Element {
    let proteins = use_get_proteins();
    let cuisines = use_get_cuisines();
    let ingredients = use_get_ingredients();
    let presets = use_get_roll_presets();
    let toast = use_toast();
    let navigator = use_navigator();

    // The shape of the week. An empty date means today; an empty seed means random.
    let mut num_days = use_signal(|| "7".to_string());
    let mut start_date = use_signal(String::new);
    let mut seed = use_signal(String::new);
    let mut rolling = use_signal(|| false);

    // Difficulty: a simple lean, or an exact per-level day mix when toggled on.
    let mut difficulty = use_signal(|| "any".to_string());
    let mut difficulty_exact = use_signal(|| false);
    let mut difficulty_mix = use_signal(default_difficulty_mix);

    // Required proteins: add one, then set how many times it should appear.
    let mut protein_picker = use_signal(Picker::default);
    let protein_rules = use_signal(Vec::<LookupRule>::new);
    use_effect(move || {
        let picked = protein_picker.read().picked.clone();
        if let Some(opt) = picked {
            append_rule(protein_rules, opt);
            protein_picker.with_mut(|p| p.type_text(String::new()));
        }
    });

    // Required cuisines: same add-and-count pattern as proteins.
    let mut cuisine_picker = use_signal(Picker::default);
    let cuisine_rules = use_signal(Vec::<LookupRule>::new);
    use_effect(move || {
        let picked = cuisine_picker.read().picked.clone();
        if let Some(opt) = picked {
            append_rule(cuisine_rules, opt);
            cuisine_picker.with_mut(|p| p.type_text(String::new()));
        }
    });

    // Ingredients the roll must use: a search field that appends to a chip list.
    let mut ing_picker = use_signal(Picker::default);
    let mut chosen_ingredients = use_signal(Vec::<SavedIngredient>::new);
    use_effect(move || {
        let picked = ing_picker.read().picked.clone();
        if let Some(opt) = picked {
            let is_dupe = chosen_ingredients.read().iter().any(|o| o.id == opt.id);
            if !is_dupe {
                chosen_ingredients.with_mut(|v| {
                    v.push(SavedIngredient {
                        id: opt.id,
                        label: opt.label.clone(),
                    })
                });
            }
            ing_picker.with_mut(|p| p.type_text(String::new()));
        }
    });

    // Presets: the current selection and the "save as" name.
    let mut preset_name = use_signal(String::new);
    let mut selected_preset = use_signal(String::new);
    let mut applied_default = use_signal(|| false);

    // On first load, apply the household's default preset if one exists.
    use_effect(move || {
        if applied_default() {
            return;
        }
        if let Some(Ok(list)) = presets() {
            if let Some(p) = list.iter().find(|p| p.is_default) {
                if let Ok(s) = serde_json::from_value::<RollSettings>(p.settings.clone()) {
                    apply_settings(
                        s,
                        num_days,
                        difficulty,
                        difficulty_exact,
                        difficulty_mix,
                        protein_rules,
                        cuisine_rules,
                        chosen_ingredients,
                    );
                    selected_preset.set(p.id.to_string());
                }
            }
            applied_default.set(true);
        }
    });

    let protein_options = lookup_options(&proteins(), |p| {
        ComboOption::new(p.protein.id, p.protein.name.clone())
    });
    let cuisine_options = lookup_options(&cuisines(), |c| {
        ComboOption::new(c.cuisine.id, c.cuisine.name.clone())
    });
    let ingredient_options = lookup_options(&ingredients(), |i| {
        ComboOption::new(i.id, i.name.clone()).with_hint(i.unit.clone())
    });

    let day_count = num_days().trim().parse::<usize>().unwrap_or(0);
    let plural = if day_count == 1 { "" } else { "s" };
    let assigned_days: u32 = difficulty_mix.read().iter().map(|d| d.days).sum();

    rsx! {
        div { class: "mr-page",
            header { class: "mr-page__head",
                Heading { level: 1, "Roll a week" }
                Text { muted: true,
                    "Set how many days to cook and any constraints, then spin. That's {day_count} dinner{plural}."
                }
            }

            VStack { gap: 1.25,

                Card { header: "Presets".to_string(),
                    VStack { gap: 0.75,
                        {
                            match presets() {
                                None => rsx! {
                                    Text { small: true, muted: true, "Loading presets…" }
                                },
                                Some(Err(e)) => rsx! {
                                    Alert { tone: Tone::Red, title: "Couldn't load presets", "{e}" }
                                },
                                Some(Ok(list)) => {
                                    let mut preset_options =
                                        vec![SelectOption::new("", "Choose a preset…")];
                                    for p in &list {
                                        let label = if p.is_default {
                                            format!("{} · default", p.name)
                                        } else {
                                            p.name.clone()
                                        };
                                        preset_options.push(SelectOption::new(p.id.to_string(), label));
                                    }
                                    let has_selection = !selected_preset().is_empty();
                                    rsx! {
                                        div { style: "display:flex; flex-wrap:wrap; gap:.6rem; align-items:flex-end;",
                                            div { style: "min-width:14rem;",
                                                Select {
                                                    label: "Load a preset".to_string(),
                                                    value: selected_preset(),
                                                    options: preset_options,
                                                    onchange: move |v| selected_preset.set(v),
                                                }
                                            }
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                disabled: !has_selection,
                                                onclick: move |_| {
                                                    let id = selected_preset();
                                                    if let Some(Ok(list)) = presets() {
                                                        if let Some(p) = list
                                                            .into_iter()
                                                            .find(|p| p.id.to_string() == id)
                                                        {
                                                            match serde_json::from_value::<
                                                                RollSettings,
                                                            >(p.settings)
                                                            {
                                                                Ok(s) => {
                                                                    apply_settings(
                                                                        s,
                                                                        num_days,
                                                                        difficulty,
                                                                        difficulty_exact,
                                                                        difficulty_mix,
                                                                        protein_rules,
                                                                        cuisine_rules,
                                                                        chosen_ingredients,
                                                                    )
                                                                }
                                                                Err(_) => {
                                                                    toast
                                                                        .show(
                                                                            Some("Load failed".to_string()),
                                                                            "This preset's settings couldn't be read.".to_string(),
                                                                            Tone::Red,
                                                                        )
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                "Load"
                                            }
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                disabled: !has_selection,
                                                onclick: move |_| {
                                                    let Ok(uid) =
                                                        Uuid::parse_str(selected_preset().as_str())
                                                    else {
                                                        return;
                                                    };
                                                    let settings = current_settings(
                                                        num_days,
                                                        difficulty,
                                                        difficulty_exact,
                                                        difficulty_mix,
                                                        protein_rules,
                                                        cuisine_rules,
                                                        chosen_ingredients,
                                                    );
                                                    let Ok(payload) = serde_json::to_value(&settings)
                                                    else {
                                                        toast
                                                            .show(
                                                                Some("Update failed".to_string()),
                                                                "Could not encode settings.".to_string(),
                                                                Tone::Red,
                                                            );
                                                        return;
                                                    };
                                                    let mut presets = presets;
                                                    spawn(async move {
                                                        match update_roll_preset(
                                                                uid,
                                                                UpdateRollPreset {
                                                                    name: None,
                                                                    settings: Some(payload),
                                                                },
                                                            )
                                                            .await
                                                        {
                                                            Ok(_) => {
                                                                presets.restart();
                                                                toast
                                                                    .show(
                                                                        Some("Updated".to_string()),
                                                                        "Preset updated.".to_string(),
                                                                        Tone::Green,
                                                                    );
                                                            }
                                                            Err(e) => {
                                                                toast
                                                                    .show(
                                                                        Some("Update failed".to_string()),
                                                                        e.to_string(),
                                                                        Tone::Red,
                                                                    )
                                                            }
                                                        }
                                                    });
                                                },
                                                "Update"
                                            }
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                disabled: !has_selection,
                                                onclick: move |_| {
                                                    let Ok(uid) =
                                                        Uuid::parse_str(selected_preset().as_str())
                                                    else {
                                                        return;
                                                    };
                                                    let mut presets = presets;
                                                    spawn(async move {
                                                        match set_default_roll_preset(uid).await {
                                                            Ok(_) => {
                                                                presets.restart();
                                                                toast
                                                                    .show(
                                                                        Some("Default set".to_string()),
                                                                        "This preset is now the default.".to_string(),
                                                                        Tone::Green,
                                                                    );
                                                            }
                                                            Err(e) => {
                                                                toast
                                                                    .show(
                                                                        Some("Couldn't set default".to_string()),
                                                                        e.to_string(),
                                                                        Tone::Red,
                                                                    )
                                                            }
                                                        }
                                                    });
                                                },
                                                "Set default"
                                            }
                                            Button {
                                                variant: ButtonVariant::Danger,
                                                disabled: !has_selection,
                                                onclick: move |_| {
                                                    let Ok(uid) =
                                                        Uuid::parse_str(selected_preset().as_str())
                                                    else {
                                                        return;
                                                    };
                                                    let mut presets = presets;
                                                    let mut selected_preset = selected_preset;
                                                    spawn(async move {
                                                        match delete_roll_preset(uid).await {
                                                            Ok(_) => {
                                                                selected_preset.set(String::new());
                                                                presets.restart();
                                                                toast
                                                                    .show(
                                                                        Some("Deleted".to_string()),
                                                                        "Preset removed.".to_string(),
                                                                        Tone::Green,
                                                                    );
                                                            }
                                                            Err(e) => {
                                                                toast
                                                                    .show(
                                                                        Some("Delete failed".to_string()),
                                                                        e.to_string(),
                                                                        Tone::Red,
                                                                    )
                                                            }
                                                        }
                                                    });
                                                },
                                                "Delete"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { style: "display:flex; flex-wrap:wrap; gap:.6rem; align-items:flex-end;",
                            div { style: "min-width:14rem;",
                                Input {
                                    label: "Save current as".to_string(),
                                    value: preset_name(),
                                    placeholder: "Preset name".to_string(),
                                    oninput: move |e: FormEvent| preset_name.set(e.value()),
                                }
                            }
                            Button {
                                variant: ButtonVariant::Solid,
                                disabled: preset_name().trim().is_empty(),
                                onclick: move |_| {
                                    let name = preset_name().trim().to_string();
                                    if name.is_empty() {
                                        return;
                                    }
                                    let settings = current_settings(
                                        num_days,
                                        difficulty,
                                        difficulty_exact,
                                        difficulty_mix,
                                        protein_rules,
                                        cuisine_rules,
                                        chosen_ingredients,
                                    );
                                    let Ok(payload) = serde_json::to_value(&settings) else {
                                        toast
                                            .show(
                                                Some("Save failed".to_string()),
                                                "Could not encode settings.".to_string(),
                                                Tone::Red,
                                            );
                                        return;
                                    };
                                    let mut presets = presets;
                                    let mut preset_name = preset_name;
                                    spawn(async move {
                                        match add_roll_preset(NewRollPreset {
                                                household_id: None,
                                                name,
                                                settings: payload,
                                            })
                                            .await
                                        {
                                            Ok(_) => {
                                                preset_name.set(String::new());
                                                presets.restart();
                                                toast
                                                    .show(
                                                        Some("Saved".to_string()),
                                                        "Preset saved.".to_string(),
                                                        Tone::Green,
                                                    );
                                            }
                                            Err(e) => {
                                                toast.show(Some("Save failed".to_string()), e.to_string(), Tone::Red)
                                            }
                                        }
                                    });
                                },
                                "Save preset"
                            }
                        }
                    }
                }

                Card { header: "Week shape".to_string(),
                    VStack { gap: 0.5,
                        div { style: "display:flex; flex-wrap:wrap; gap:.6rem; align-items:flex-end;",
                            div { style: "width:8rem;",
                                Input {
                                    label: "How many days".to_string(),
                                    kind: "number".to_string(),
                                    value: num_days(),
                                    oninput: move |e: FormEvent| num_days.set(e.value()),
                                }
                            }
                            div { style: "width:11rem;",
                                Input {
                                    label: "Start date".to_string(),
                                    kind: "date".to_string(),
                                    value: start_date(),
                                    oninput: move |e: FormEvent| start_date.set(e.value()),
                                }
                            }
                            div { style: "width:9rem;",
                                Input {
                                    label: "Seed".to_string(),
                                    kind: "number".to_string(),
                                    value: seed(),
                                    placeholder: "Random".to_string(),
                                    oninput: move |e: FormEvent| seed.set(e.value()),
                                }
                            }
                        }
                        Text { small: true, muted: true,
                            "Leave the date empty to start today. Enter a plan's seed to repeat that roll."
                        }
                    }
                }

                Card { header: "Difficulty".to_string(),
                    VStack { gap: 0.75,
                        Text { small: true, muted: true,
                            "Saved with presets. The roller doesn't apply difficulty yet."
                        }
                        Switch {
                            checked: difficulty_exact(),
                            onchange: move |v| difficulty_exact.set(v),
                            label: "Set an exact day mix".to_string(),
                        }
                        if difficulty_exact() {
                            div { style: "display:flex; flex-wrap:wrap; gap:.6rem; align-items:flex-end;",
                                for (i, dd) in difficulty_mix.read().clone().into_iter().enumerate() {
                                    {
                                        let idx = i;
                                        rsx! {
                                            div { style: "width:6.5rem;",
                                                Input {
                                                    label: difficulty_label(dd.level.into())
                                                                                                            .to_string(),
                                                    kind: "number".to_string(),
                                                    value: "{dd.days}",
                                                    oninput: move |e: FormEvent| {
                                                        let v = e.value().parse::<u32>().unwrap_or(0);
                                                        difficulty_mix
                                                            .with_mut(|m| {
                                                                if let Some(x) = m.get_mut(idx) {
                                                                    x.days = v;
                                                                }
                                                            });
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Text { small: true, muted: true,
                                "{assigned_days} of {day_count} day{plural} set. Any unassigned days roll at any difficulty."
                            }
                        } else {
                            Select {
                                label: "Overall lean".to_string(),
                                value: difficulty(),
                                options: vec![
                                    SelectOption::new("any", "No preference"),
                                    SelectOption::new("easy", "Easier meals"),
                                    SelectOption::new("balanced", "Balanced"),
                                    SelectOption::new("ambitious", "More ambitious"),
                                ],
                                onchange: move |v| difficulty.set(v),
                            }
                        }
                    }
                }

                Card { header: "Proteins".to_string(),
                    VStack { gap: 0.75,
                        Text { small: true, muted: true,
                            "Add a protein to require it, then set how many times it should appear."
                        }
                        {
                            match proteins() {
                                Some(Ok(_)) => rsx! {
                                    Combobox {
                                        label: "Add a protein".to_string(),
                                        placeholder: "Search proteins…".to_string(),
                                        options: protein_options,
                                        picker: protein_picker,
                                        can_create: false,
                                    }
                                },
                                Some(Err(e)) => rsx! {
                                    Alert { tone: Tone::Red, title: "Couldn't load proteins", "{e}" }
                                },
                                None => rsx! {
                                    Text { small: true, muted: true, "Loading proteins…" }
                                },
                            }
                        }
                        for rule in protein_rules.read().clone() {
                            RuleRow {
                                key: "{rule.id}",
                                rule: rule.clone(),
                                rules: protein_rules,
                            }
                        }
                    }
                }

                Card { header: "Cuisines".to_string(),
                    VStack { gap: 0.75,
                        Text { small: true, muted: true,
                            "Add a cuisine to require it, then set how many times it should appear."
                        }
                        {
                            match cuisines() {
                                Some(Ok(_)) => rsx! {
                                    Combobox {
                                        label: "Add a cuisine".to_string(),
                                        placeholder: "Search cuisines…".to_string(),
                                        options: cuisine_options,
                                        picker: cuisine_picker,
                                        can_create: false,
                                    }
                                },
                                Some(Err(e)) => rsx! {
                                    Alert { tone: Tone::Red, title: "Couldn't load cuisines", "{e}" }
                                },
                                None => rsx! {
                                    Text { small: true, muted: true, "Loading cuisines…" }
                                },
                            }
                        }
                        for rule in cuisine_rules.read().clone() {
                            RuleRow {
                                key: "{rule.id}",
                                rule: rule.clone(),
                                rules: cuisine_rules,
                            }
                        }
                    }
                }

                Card { header: "Must-use ingredients".to_string(),
                    VStack { gap: 0.75,
                        Text { small: true, muted: true,
                            "Guarantee the week includes a recipe using each of these. Saved with presets; the roller doesn't apply ingredients yet."
                        }
                        {
                            match ingredients() {
                                Some(Ok(_)) => rsx! {
                                    Combobox {
                                        label: "Add an ingredient".to_string(),
                                        placeholder: "Search ingredients…".to_string(),
                                        options: ingredient_options,
                                        picker: ing_picker,
                                        can_create: false,
                                    }
                                },
                                Some(Err(e)) => rsx! {
                                    Alert { tone: Tone::Red, title: "Couldn't load ingredients", "{e}" }
                                },
                                None => rsx! {
                                    Text { small: true, muted: true, "Loading ingredients…" }
                                },
                            }
                        }
                        if !chosen_ingredients.read().is_empty() {
                            div { style: "display:flex; flex-wrap:wrap; gap:.5rem;",
                                for opt in chosen_ingredients.read().clone() {
                                    {
                                        let iid = opt.id;
                                        rsx! {
                                            Chip {
                                                key: "{opt.id}",
                                                label: opt.label.clone(),
                                                onremove: move |_| { chosen_ingredients.with_mut(|v| v.retain(|o| o.id != iid)) },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "display:flex; align-items:center; gap:.9rem; flex-wrap:wrap;",
                    Button {
                        variant: ButtonVariant::Solid,
                        size: Size::Lg,
                        disabled: day_count == 0,
                        loading: rolling(),
                        onclick: move |_| {
                            let Ok(num_days) = u16::try_from(day_count) else {
                                return;
                            };
                            if num_days == 0 || rolling() {
                                return;
                            }
                            let constraints = RollConstraints {
                                cuisines: count_rules(&cuisine_rules.read()),
                                proteins: count_rules(&protein_rules.read()),
                            };
                            let seed = seed().trim().parse::<i64>().ok();
                            let typed_date = start_date();
                            rolling.set(true);
                            spawn(async move {
                                let start_date = match typed_date.trim() {
                                    "" => today().await,
                                    typed => {
                                        match parse_iso_date(typed) {
                                            Some(date) => Some(date),
                                            None => {
                                                toast
                                                    .show(
                                                        Some("Check the date".to_string()),
                                                        "The start date couldn't be read.".to_string(),
                                                        Tone::Red,
                                                    );
                                                rolling.set(false);
                                                return;
                                            }
                                        }
                                    }
                                };
                                let request = RollRequest {
                                    household_id: None,
                                    start_date,
                                    num_days,
                                    seed,
                                    constraints,
                                };
                                match roll_week(request).await {
                                    Ok(plan) => {
                                        toast
                                            .show(
                                                Some("Rolled".to_string()),
                                                format!("{} dinners planned.", plan.meals.len()),
                                                Tone::Green,
                                            );
                                        navigator.push(Route::PlannedMeals {});
                                    }
                                    Err(e) => {
                                        toast
                                            .show(
                                                Some("Couldn't roll".to_string()),
                                                e.to_string(),
                                                Tone::Red,
                                            )
                                    }
                                }
                                rolling.set(false);
                            });
                        },
                        "Roll the week"
                    }
                    if day_count == 0 {
                        Text { small: true, muted: true, "Enter how many days to cook." }
                    }
                }
            }
        }
    }
}

/// The rules as the solver wants them: ids and day counts, no display names.
fn count_rules(rules: &[LookupRule]) -> Vec<CountRule> {
    rules
        .iter()
        .map(|rule| CountRule {
            id: rule.id,
            times: rule.times,
        })
        .collect()
}

/// The browser's local date, so a plan starts on the user's today rather than
/// the server's. `None` lets the server fall back to its own date.
async fn today() -> Option<time::Date> {
    let script = "const d = new Date(); \
        return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;";
    let value = document::eval(script).await.ok()?;
    parse_iso_date(value.as_str()?)
}

/// Append a picked lookup option as a rule (count 1), skipping duplicates.
fn append_rule(mut rules: Signal<Vec<LookupRule>>, opt: ComboOption) {
    if rules.read().iter().any(|r| r.id == opt.id) {
        return;
    }
    rules.with_mut(|v| {
        v.push(LookupRule {
            id: opt.id,
            name: opt.label.clone(),
            times: 1,
        })
    });
}

/// One required protein/cuisine row: the name, a target count, and a remove
/// control. Edits write back into the shared `rules` list.
#[component]
fn RuleRow(rule: LookupRule, mut rules: Signal<Vec<LookupRule>>) -> Element {
    let rid = rule.id;
    rsx! {
        div { style: "display:flex; align-items:flex-end; gap:.6rem; flex-wrap:wrap;",
            span { style: "min-width:6rem; padding-bottom:.55rem; font-weight:600;",
                "{rule.name}"
            }
            div { style: "width:5.5rem;",
                Input {
                    label: "Times".to_string(),
                    kind: "number".to_string(),
                    value: "{rule.times}",
                    oninput: move |e: FormEvent| {
                        let v = e.value().parse::<u32>().unwrap_or(1).max(1);
                        rules
                            .with_mut(|list| {
                                if let Some(r) = list.iter_mut().find(|r| r.id == rid) {
                                    r.times = v;
                                }
                            });
                    },
                }
            }
            Button {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                onclick: move |_| rules.with_mut(|list| list.retain(|r| r.id != rid)),
                "Remove"
            }
        }
    }
}

/// Read the whole page state into a `RollSettings` for saving.
fn current_settings(
    num_days: Signal<String>,
    difficulty: Signal<String>,
    difficulty_exact: Signal<bool>,
    difficulty_mix: Signal<Vec<DifficultyDays>>,
    protein_rules: Signal<Vec<LookupRule>>,
    cuisine_rules: Signal<Vec<LookupRule>>,
    chosen_ingredients: Signal<Vec<SavedIngredient>>,
) -> RollSettings {
    RollSettings {
        num_days: num_days(),
        difficulty: difficulty(),
        difficulty_exact: difficulty_exact(),
        difficulty_mix: difficulty_mix(),
        proteins: protein_rules(),
        cuisines: cuisine_rules(),
        ingredients: chosen_ingredients(),
    }
}

/// Load a `RollSettings` back into the page's signals.
#[allow(clippy::too_many_arguments)]
fn apply_settings(
    s: RollSettings,
    mut num_days: Signal<String>,
    mut difficulty: Signal<String>,
    mut difficulty_exact: Signal<bool>,
    mut difficulty_mix: Signal<Vec<DifficultyDays>>,
    mut protein_rules: Signal<Vec<LookupRule>>,
    mut cuisine_rules: Signal<Vec<LookupRule>>,
    mut chosen_ingredients: Signal<Vec<SavedIngredient>>,
) {
    num_days.set(s.num_days);
    difficulty.set(s.difficulty);
    difficulty_exact.set(s.difficulty_exact);
    difficulty_mix.set(if s.difficulty_mix.is_empty() {
        default_difficulty_mix()
    } else {
        s.difficulty_mix
    });
    protein_rules.set(s.proteins);
    cuisine_rules.set(s.cuisines);
    chosen_ingredients.set(s.ingredients);
}
