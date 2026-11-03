//! The roll setup page: pick a preset (or build fresh), choose how many days
//! and the constraints, then hit Roll. Rolling sends every rule to the solver
//! and lands on the planned-meals page. Presets save the
//! whole form as a JSON payload through the roll_preset API.

use crate::components::ui::*;
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::roll::components::DayLimitsCard;
use crate::features::roll::hooks::use_roll::roll_week;
use crate::features::roll::hooks::use_roll_preset::{
    add_roll_preset, delete_roll_preset, set_default_roll_preset, update_roll_preset,
    use_get_roll_presets,
};
use crate::features::roll::settings::{
    band_label, bound_label, importance_label, DayLimitSettings, LookupRule, RollSettings,
    DEFAULT_NUM_DAYS, IMPORTANCE_LEVELS,
};
use crate::hooks::use_toast::use_toast;
use crate::models::{error_message, parse_iso_date};
use crate::router::Route;
use dioxus::prelude::*;
use domain::roll::{
    Bound, CountRule, DifficultyBand, DifficultyRule, Importance, RollConstraints, RollOutcome,
    RollRequest, RuleFamily,
};
use domain::roll_preset::{NewRollPreset, UpdateRollPreset};
use uuid::Uuid;

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
    let mut num_days = use_signal(|| DEFAULT_NUM_DAYS.to_string());
    let mut start_date = use_signal(String::new);
    let mut seed = use_signal(String::new);
    let mut rolling = use_signal(|| false);

    // Difficulty: how many dinners land in each band.
    let mut difficulty_rules = use_signal(Vec::<DifficultyRule>::new);
    let day_limits = use_signal(DayLimitSettings::default);

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

    // Ingredients the roll must use: same add-and-count pattern, counting the
    // dinners whose recipe uses the ingredient.
    let mut ing_picker = use_signal(Picker::default);
    let ingredient_rules = use_signal(Vec::<LookupRule>::new);
    use_effect(move || {
        let picked = ing_picker.read().picked.clone();
        if let Some(opt) = picked {
            append_rule(ingredient_rules, opt);
            ing_picker.with_mut(|p| p.type_text(String::new()));
        }
    });

    let form = RollForm {
        num_days,
        difficulties: difficulty_rules,
        proteins: protein_rules,
        cuisines: cuisine_rules,
        ingredients: ingredient_rules,
        day_limits,
    };

    // Presets: the current selection and the "save as" name.
    let mut preset_name = use_signal(String::new);
    let mut selected_preset = use_signal(String::new);
    let mut applied_default = use_signal(|| false);
    // The preset awaiting delete confirmation: its id and name.
    let mut confirm_preset_delete = use_signal(|| None as Option<(Uuid, String)>);

    let mut ask_delete_preset = move || {
        let Ok(uid) = Uuid::parse_str(selected_preset().as_str()) else {
            return;
        };
        let name = match presets() {
            Some(Ok(list)) => list
                .iter()
                .find(|p| p.id == uid)
                .map(|p| p.name.clone())
                .unwrap_or_default(),
            _ => String::new(),
        };
        confirm_preset_delete.set(Some((uid, name)));
    };

    let delete_preset = move |uid: Uuid| {
        let mut presets = presets;
        spawn(async move {
            match delete_roll_preset(uid).await {
                Ok(_) => {
                    selected_preset.set(String::new());
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
            confirm_preset_delete.set(None);
        });
    };

    // On first load, apply the household's default preset if one exists.
    use_effect(move || {
        if applied_default() {
            return;
        }
        if let Some(Ok(list)) = presets() {
            if let Some(p) = list.iter().find(|p| p.is_default) {
                if let Ok(s) = serde_json::from_value::<RollSettings>(p.settings.clone()) {
                    apply_settings(s, form);
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

    rsx! {
        div { class: "mr-page",
            if let Some((uid, name)) = confirm_preset_delete() {
                ConfirmDialog {
                    title: format!("Delete the preset “{name}”?"),
                    confirm_label: "Delete".to_string(),
                    danger: true,
                    on_confirm: move |_| delete_preset(uid),
                    on_cancel: move |_| confirm_preset_delete.set(None),
                    Text { "Your saved rolls are not affected." }
                }
            }
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
                                    Alert { tone: Tone::Red, title: "Couldn't load presets", "{error_message(&e)}" }
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
                                        div { class: "mr-fieldrow",
                                            div { class: "mr-fieldrow__wide",
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
                                                                    apply_settings(s, form)
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
                                                    let settings = current_settings(form);
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
                                                                        error_message(&e),
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
                                                                        error_message(&e),
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
                                                onclick: move |_| ask_delete_preset(),
                                                "Delete"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "mr-fieldrow",
                            div { class: "mr-fieldrow__wide",
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
                                    let settings = current_settings(form);
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
                                                toast.show(Some("Save failed".to_string()), error_message(&e), Tone::Red)
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
                        div { class: "mr-fieldrow",
                            div { class: "mr-fieldw mr-fieldw--count",
                                Input {
                                    label: "How many days".to_string(),
                                    kind: "number".to_string(),
                                    value: num_days(),
                                    oninput: move |e: FormEvent| num_days.set(e.value()),
                                }
                            }
                            div { class: "mr-fieldw mr-fieldw--date",
                                Input {
                                    label: "Start date".to_string(),
                                    kind: "date".to_string(),
                                    value: start_date(),
                                    oninput: move |e: FormEvent| start_date.set(e.value()),
                                }
                            }
                            div { class: "mr-fieldw mr-fieldw--seed",
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
                            "Count dinners by difficulty, e.g. at least 3 easy and at most 1 hard. Every rule is Required unless you lower it: High, Medium and Low rules are preferences, kept most important first as far as your recipes allow."
                        }
                        Select {
                            label: "Add a difficulty rule".to_string(),
                            value: String::new(),
                            placeholder: "Choose a band…".to_string(),
                            options: DifficultyBand::ALL
                                .into_iter()
                                .filter(|band| !difficulty_rules.read().iter().any(|r| r.band == *band))
                                .map(|band| SelectOption::new(format!("{band:?}"), band_label(band)))
                                .collect::<Vec<_>>(),
                            onchange: move |value: String| add_difficulty_rule(difficulty_rules, &value),
                        }
                        for (index, rule) in difficulty_rules.read().clone().into_iter().enumerate() {
                            RuleRow {
                                key: "{rule.band:?}",
                                name: band_label(rule.band).to_string(),
                                times: rule.times,
                                bound: rule.bound,
                                importance: rule.importance,
                                on_change: move |(times, bound, importance): (u32, Bound, Importance)| {
                                    if let Some(r) = difficulty_rules.write().get_mut(index) {
                                        r.times = times;
                                        r.bound = bound;
                                        r.importance = importance;
                                    }
                                },
                                on_remove: move |_| {
                                    difficulty_rules.write().remove(index);
                                },
                            }
                        }
                    }
                }

                DayLimitsCard { limits: day_limits, day_count }

                Card { header: "Proteins".to_string(),
                    VStack { gap: 0.75,
                        Text { small: true, muted: true,
                            "Add a protein, then set how often it should (or may) appear."
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
                                    Alert { tone: Tone::Red, title: "Couldn't load proteins", "{error_message(&e)}" }
                                },
                                None => rsx! {
                                    Text { small: true, muted: true, "Loading proteins…" }
                                },
                            }
                        }
                        LookupRules { rules: protein_rules }
                    }
                }

                Card { header: "Cuisines".to_string(),
                    VStack { gap: 0.75,
                        Text { small: true, muted: true,
                            "Add a cuisine, then set how often it should (or may) appear."
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
                                    Alert { tone: Tone::Red, title: "Couldn't load cuisines", "{error_message(&e)}" }
                                },
                                None => rsx! {
                                    Text { small: true, muted: true, "Loading cuisines…" }
                                },
                            }
                        }
                        LookupRules { rules: cuisine_rules }
                    }
                }

                Card { header: "Must-use ingredients".to_string(),
                    VStack { gap: 0.75,
                        Text { small: true, muted: true,
                            "Count dinners whose recipe uses an ingredient, e.g. at least 1 that uses up the spinach."
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
                                    Alert { tone: Tone::Red, title: "Couldn't load ingredients", "{error_message(&e)}" }
                                },
                                None => rsx! {
                                    Text { small: true, muted: true, "Loading ingredients…" }
                                },
                            }
                        }
                        LookupRules { rules: ingredient_rules }
                    }
                }

                div { class: "mr-rollactions",
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
                                ingredients: count_rules(&ingredient_rules.read()),
                                difficulties: difficulty_rules(),
                                day_limits: day_limits().plan(usize::from(num_days)),
                            };
                            let names = RuleNames::of(form);
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
                                    Ok(outcome) => {
                                        let (title, message, tone) = names.summary(&outcome);
                                        toast.show(Some(title), message, tone);
                                        navigator.push(Route::PlannedMeals {});
                                    }
                                    Err(e) => {
                                        toast
                                            .show(
                                                Some("Couldn't roll".to_string()),
                                                error_message(&e),
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

/// The rules as the solver wants them: ids and counts, no display names.
fn count_rules(rules: &[LookupRule]) -> Vec<CountRule> {
    rules
        .iter()
        .map(|rule| CountRule {
            id: rule.id,
            times: rule.times,
            bound: rule.bound,
            importance: rule.importance,
        })
        .collect()
}

/// Each rule list's display names, captured at roll time so the result can
/// name the preferences it had to loosen.
struct RuleNames {
    difficulties: Vec<String>,
    proteins: Vec<String>,
    cuisines: Vec<String>,
    ingredients: Vec<String>,
}

impl RuleNames {
    fn of(form: RollForm) -> Self {
        let names =
            |rules: Signal<Vec<LookupRule>>| rules.read().iter().map(|r| r.name.clone()).collect();
        RuleNames {
            difficulties: form
                .difficulties
                .read()
                .iter()
                .map(|r| band_label(r.band).to_string())
                .collect(),
            proteins: names(form.proteins),
            cuisines: names(form.cuisines),
            ingredients: names(form.ingredients),
        }
    }

    /// The toast after a roll: plain success, or which preferences fell short.
    fn summary(&self, outcome: &RollOutcome) -> (String, String, Tone) {
        let planned = format!("{} dinners planned.", outcome.plan.meals.len());
        if outcome.unmet.is_empty() {
            return ("Rolled".to_string(), planned, Tone::Green);
        }
        let short: Vec<String> = outcome
            .unmet
            .iter()
            .map(|u| {
                let list = match u.family {
                    RuleFamily::Difficulty => &self.difficulties,
                    RuleFamily::Protein => &self.proteins,
                    RuleFamily::Cuisine => &self.cuisines,
                    RuleFamily::Ingredient => &self.ingredients,
                };
                let name = list.get(u.index).map_or("A rule", String::as_str);
                format!(
                    "{name}: wanted {} {}, got {}",
                    bound_label(u.bound).to_lowercase(),
                    u.times,
                    u.got
                )
            })
            .collect();
        (
            "Rolled, with some preferences loosened".to_string(),
            format!("{planned} {}.", short.join("; ")),
            Tone::Amber,
        )
    }
}

/// The browser's local date, so a plan starts on the user's today rather than
/// the server's. `None` lets the server fall back to its own date.
async fn today() -> Option<time::Date> {
    let script = "const d = new Date(); \
        return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;";
    let value = document::eval(script).await.ok()?;
    parse_iso_date(value.as_str()?)
}

/// Append a picked lookup option as a rule (at least once, required), skipping duplicates.
fn append_rule(mut rules: Signal<Vec<LookupRule>>, opt: ComboOption) {
    if rules.read().iter().any(|r| r.id == opt.id) {
        return;
    }
    rules.with_mut(|v| {
        v.push(LookupRule {
            id: opt.id,
            name: opt.label.clone(),
            times: 1,
            bound: Bound::AtLeast,
            importance: Importance::Required,
        })
    });
}

fn add_difficulty_rule(mut rules: Signal<Vec<DifficultyRule>>, value: &str) {
    let Some(band) = DifficultyBand::ALL
        .into_iter()
        .find(|band| format!("{band:?}") == value)
    else {
        return;
    };
    if rules.read().iter().any(|r| r.band == band) {
        return;
    }
    rules.write().push(DifficultyRule {
        band,
        times: 1,
        bound: Bound::AtLeast,
        importance: Importance::Required,
    });
}

/// Every rule list on the page, so presets read and write them in one go.
#[derive(Clone, Copy)]
struct RollForm {
    num_days: Signal<String>,
    difficulties: Signal<Vec<DifficultyRule>>,
    proteins: Signal<Vec<LookupRule>>,
    cuisines: Signal<Vec<LookupRule>>,
    ingredients: Signal<Vec<LookupRule>>,
    day_limits: Signal<DayLimitSettings>,
}

/// The rows for one protein / cuisine / ingredient list; edits write back into it.
#[component]
fn LookupRules(mut rules: Signal<Vec<LookupRule>>) -> Element {
    rsx! {
        for rule in rules.read().clone() {
            {
                let rid = rule.id;
                rsx! {
                    RuleRow {
                        key: "{rule.id}",
                        name: rule.name.clone(),
                        times: rule.times,
                        bound: rule.bound,
                        importance: rule.importance,
                        on_change: move |(times, bound, importance): (u32, Bound, Importance)| {
                            if let Some(r) = rules.write().iter_mut().find(|r| r.id == rid) {
                                r.times = times;
                                r.bound = bound;
                                r.importance = importance;
                            }
                        },
                        on_remove: move |_| rules.write().retain(|r| r.id != rid),
                    }
                }
            }
        }
    }
}

/// One rule: its name, at least / at most, a count, how much it matters, and a
/// remove control. Every edit reports the whole (count, bound, importance) triple.
#[component]
fn RuleRow(
    name: String,
    times: u32,
    bound: Bound,
    importance: Importance,
    on_change: EventHandler<(u32, Bound, Importance)>,
    on_remove: EventHandler<()>,
) -> Element {
    let bound_options = vec![
        SelectOption::new("AtLeast", bound_label(Bound::AtLeast)),
        SelectOption::new("AtMost", bound_label(Bound::AtMost)),
    ];
    let importance_options: Vec<SelectOption> = IMPORTANCE_LEVELS
        .into_iter()
        .map(|level| SelectOption::new(format!("{level:?}"), importance_label(level)))
        .collect();
    // "At most 0" means "none of these"; "at least 0" would ask for nothing.
    let floor = match bound {
        Bound::AtLeast => 1,
        Bound::AtMost => 0,
    };

    rsx! {
        div { class: "mr-fieldrow",
            span { class: "mr-rulename",
                "{name}"
            }
            div { class: "mr-fieldw mr-fieldw--count",
                Select {
                    label: "Count".to_string(),
                    value: format!("{bound:?}"),
                    options: bound_options,
                    onchange: move |value: String| {
                        let bound = if value == "AtMost" { Bound::AtMost } else { Bound::AtLeast };
                        let floor = if bound == Bound::AtMost { 0 } else { 1 };
                        on_change.call((times.max(floor), bound, importance));
                    },
                }
            }
            div { class: "mr-fieldw mr-fieldw--times",
                Input {
                    label: "Times".to_string(),
                    kind: "number".to_string(),
                    value: "{times}",
                    oninput: move |e: FormEvent| {
                        let v = e.value().parse::<u32>().unwrap_or(floor).max(floor);
                        on_change.call((v, bound, importance));
                    },
                }
            }
            div { class: "mr-fieldw mr-fieldw--importance",
                Select {
                    label: "Importance".to_string(),
                    value: format!("{importance:?}"),
                    options: importance_options,
                    onchange: move |value: String| {
                        let level = IMPORTANCE_LEVELS
                            .into_iter()
                            .find(|level| format!("{level:?}") == value)
                            .unwrap_or_default();
                        on_change.call((times, bound, level));
                    },
                }
            }
            Button {
                variant: ButtonVariant::Ghost,
                size: Size::Sm,
                onclick: move |_| on_remove.call(()),
                "Remove"
            }
        }
    }
}

/// Read the whole page state into a `RollSettings` for saving.
fn current_settings(form: RollForm) -> RollSettings {
    RollSettings {
        num_days: (form.num_days)(),
        difficulties: (form.difficulties)(),
        proteins: (form.proteins)(),
        cuisines: (form.cuisines)(),
        ingredients: (form.ingredients)(),
        day_limits: (form.day_limits)(),
        ..RollSettings::default()
    }
}

/// Load a `RollSettings` back into the page's signals.
fn apply_settings(s: RollSettings, mut form: RollForm) {
    form.difficulties.set(s.difficulty_rules());
    form.num_days.set(s.num_days);
    form.proteins.set(s.proteins);
    form.cuisines.set(s.cuisines);
    form.ingredients.set(s.ingredients);
    form.day_limits.set(s.day_limits);
}
