//! The roll page: choose how many dinners, add any rules, and roll. The roll is a
//! preview that shows in place; Keep stores it as a plan, so rolling again and
//! again leaves nothing behind in History.

use crate::components::ui::*;
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::roll::components::{
    ConflictAlert, DayLimitsCard, PresetBar, RollBar, RollErrorAlert, RollHero, RollResults,
    RolledWeek, RuleChoice, RulesPanel,
};
use crate::features::roll::hooks::use_fit_count::use_fit_count;
use crate::features::roll::hooks::use_roll::{keep_roll, preview_roll, reroll_preview};
use crate::features::roll::settings::{
    band_label, RollSettings, RuleTarget, DEFAULT_NUM_DAYS, MAX_ROLL_DAYS,
};
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use crate::router::Route;
use dioxus::prelude::*;
use domain::roll::{
    DifficultyBand, FitRequest, KeepRollRequest, RerollPreviewRequest, RollConstraints,
    RollRequest, RuleFamily,
};

/// A roll on screen with what it was rolled under, so Keep and the per-day
/// re-roll use the same rules even if the form has been edited since.
#[derive(Clone, PartialEq)]
struct Shown {
    week: RolledWeek,
    settings: RollSettings,
    constraints: RollConstraints,
    recipe_ids: Vec<uuid::Uuid>,
}

#[component]
pub fn Roll() -> Element {
    let proteins = use_get_proteins();
    let cuisines = use_get_cuisines();
    let ingredients = use_get_ingredients();
    let toast = use_toast();
    let navigator = use_navigator();

    let mut settings = use_signal(|| RollSettings {
        num_days: DEFAULT_NUM_DAYS.to_string(),
        ..RollSettings::default()
    });
    // An empty seed means random.
    let mut seed = use_signal(String::new);

    let mut rules_open = use_signal(|| false);
    let mut limits_open = use_signal(|| false);
    let mut advanced_open = use_signal(|| false);

    let mut shown = use_signal(|| None as Option<Shown>);
    let mut roll_error = use_signal(|| None as Option<String>);
    let mut rolling = use_signal(|| false);
    let mut rerolling = use_signal(|| None as Option<usize>);
    let mut keeping = use_signal(|| false);

    let days = use_memo(move || settings.read().days());
    let fit_request = use_memo(move || {
        let days = days();
        (1..=MAX_ROLL_DAYS).contains(&days).then(|| FitRequest {
            num_days: days as u16,
            constraints: settings.read().constraints(days as usize),
        })
    });
    let fit = use_fit_count(fit_request);

    let conflicts = settings.read().conflicts(days());
    let fit_count = match fit() {
        Some(Some(Ok(count))) => Some(count),
        _ => None,
    };
    let too_few = fit_count.is_some_and(|c| c.fitting < days() && c.total > 0);
    let note = match fit_count {
        Some(c) if c.total == 0 => Some((
            "There are no recipes to roll from yet. Add some under Recipes.".to_string(),
            true,
        )),
        Some(c) if too_few => Some((
            format!(
                "Only {} recipe{} fit your Must rules and day limits, but you are rolling {}.",
                c.fitting,
                if c.fitting == 1 { "" } else { "s" },
                days()
            ),
            true,
        )),
        Some(c) => Some((
            format!("{} of {} recipes fit your Must rules.", c.fitting, c.total),
            false,
        )),
        None => None,
    };
    let valid_days = (1..=MAX_ROLL_DAYS).contains(&days());
    let can_roll = valid_days && conflicts.is_empty() && !too_few;

    let mut choices: Vec<RuleChoice> = DifficultyBand::ALL
        .into_iter()
        .map(|band| RuleChoice {
            label: format!("{} · difficulty", band_label(band)),
            target: RuleTarget::Band(band),
        })
        .collect();
    let mut load_failed = false;
    let mut add_lookups =
        |family: RuleFamily, noun: &str, rows: Option<Vec<(uuid::Uuid, String)>>| match rows {
            Some(rows) => choices.extend(rows.into_iter().map(|(id, name)| RuleChoice {
                label: format!("{name} · {noun}"),
                target: RuleTarget::Lookup { family, id, name },
            })),
            None => load_failed = true,
        };
    add_lookups(
        RuleFamily::Protein,
        "protein",
        match proteins() {
            Some(Ok(list)) => Some(
                list.into_iter()
                    .map(|p| (p.protein.id, p.protein.name))
                    .collect(),
            ),
            _ => None,
        },
    );
    add_lookups(
        RuleFamily::Cuisine,
        "cuisine",
        match cuisines() {
            Some(Ok(list)) => Some(
                list.into_iter()
                    .map(|c| (c.cuisine.id, c.cuisine.name))
                    .collect(),
            ),
            _ => None,
        },
    );
    add_lookups(
        RuleFamily::Ingredient,
        "ingredient",
        match ingredients() {
            Some(Ok(list)) => Some(list.into_iter().map(|i| (i.id, i.name)).collect()),
            _ => None,
        },
    );
    let still_loading = proteins().is_none() || cuisines().is_none() || ingredients().is_none();
    let load_error = (load_failed && !still_loading)
        .then(|| "Some proteins, cuisines or ingredients could not be loaded, so they are missing from the search.".to_string());

    let do_roll = move |_| {
        if rolling() || keeping() {
            return;
        }
        let form = settings();
        let days = form.days();
        if !(1..=MAX_ROLL_DAYS).contains(&days) {
            return;
        }
        let constraints = form.constraints(days as usize);
        let seed = seed().trim().parse::<i64>().ok();
        rolling.set(true);
        roll_error.set(None);
        spawn(async move {
            let request = RollRequest {
                household_id: None,
                start_date: None,
                num_days: days as u16,
                seed,
                constraints: constraints.clone(),
            };
            match preview_roll(request).await {
                Ok(preview) => {
                    let recipe_ids = preview.meals.iter().map(|m| m.recipe_id).collect();
                    shown.set(Some(Shown {
                        week: RolledWeek {
                            seed: preview.seed,
                            unmet: form.unmet_sentences(&preview.unmet),
                            meals: preview.meals,
                        },
                        settings: form,
                        constraints,
                        recipe_ids,
                    }));
                }
                Err(e) => {
                    roll_error.set(Some(error_message(&e)));
                    rules_open.set(true);
                }
            }
            rolling.set(false);
        });
    };

    let do_reroll = move |day: usize| {
        let Some(current) = shown() else {
            return;
        };
        if rerolling().is_some() {
            return;
        }
        rerolling.set(Some(day));
        roll_error.set(None);
        spawn(async move {
            let request = RerollPreviewRequest {
                constraints: current.constraints.clone(),
                recipe_ids: current.recipe_ids.clone(),
                day: day as u16,
            };
            match reroll_preview(request).await {
                Ok(replaced) => {
                    let mut next = current;
                    if let Some(slot) = next.week.meals.get_mut(day) {
                        *slot = replaced.meal;
                    }
                    next.recipe_ids = next.week.meals.iter().map(|m| m.recipe_id).collect();
                    next.week.unmet = next.settings.unmet_sentences(&replaced.unmet);
                    shown.set(Some(next));
                }
                Err(e) => {
                    roll_error.set(Some(error_message(&e)));
                }
            }
            rerolling.set(None);
        });
    };

    let do_keep = move |_| {
        let Some(current) = shown() else {
            return;
        };
        if keeping() {
            return;
        }
        keeping.set(true);
        roll_error.set(None);
        spawn(async move {
            let request = KeepRollRequest {
                household_id: None,
                seed: Some(current.week.seed),
                constraints: current.constraints,
                recipe_ids: current.recipe_ids,
            };
            match keep_roll(request).await {
                Ok(_) => {
                    toast.show(
                        Some("Kept".to_string()),
                        "The week is saved. Put it on days when you are ready.".to_string(),
                        Tone::Green,
                    );
                    navigator.push(Route::PlannedMeals {});
                }
                Err(e) => {
                    roll_error.set(Some(error_message(&e)));
                    keeping.set(false);
                }
            }
        });
    };

    let (note_text, note_warns) = note.unzip();

    rsx! {
        div { class: "mr-page mr-roll",
            RollHero {
                days: days,
                on_days: move |n: u32| settings.write().num_days = n.to_string(),
                spinning: rolling(),
                PresetBar { settings }
            }

            if !conflicts.is_empty() {
                ConflictAlert {
                    conflicts: conflicts.clone(),
                    on_loosen: {
                        let conflicts = conflicts.clone();
                        move |_| {
                            let rules: Vec<_> = conflicts.iter().flat_map(|c| c.rules.clone()).collect();
                            settings.write().make_nice_to_have(&rules);
                        }
                    },
                }
            }
            if let Some(message) = roll_error() {
                RollErrorAlert { message, on_dismiss: move |_| roll_error.set(None) }
            }

            RollBar {
                rolling: rolling(),
                disabled: !can_roll || keeping(),
                on_roll: do_roll,
                note: note_text,
                note_warns: note_warns.unwrap_or(false),
            }

            if let Some(current) = shown() {
                RollResults {
                    week: current.week,
                    rerolling: rerolling(),
                    rolling: rolling(),
                    keeping: keeping(),
                    on_keep: do_keep,
                    on_again: do_roll,
                    on_reroll: do_reroll,
                }
            }

            VStack { gap: 0.9,
                RulesPanel {
                    settings,
                    choices,
                    open: rules_open(),
                    ontoggle: move |open| rules_open.set(open),
                    load_error,
                }
                Collapsible {
                    title: "Day limits".to_string(),
                    summary: if settings.read().day_limits.enabled { "On".to_string() } else { "Off".to_string() },
                    open: limits_open(),
                    ontoggle: move |open| limits_open.set(open),
                    DayLimitsCard { settings, day_count: days() as usize }
                }
                Collapsible {
                    title: "Advanced".to_string(),
                    open: advanced_open(),
                    ontoggle: move |open| advanced_open.set(open),
                    div { class: "mr-roll__seed",
                        Input {
                            label: "Seed".to_string(),
                            kind: "number".to_string(),
                            value: seed(),
                            placeholder: "Random".to_string(),
                            oninput: move |e: FormEvent| seed.set(e.value()),
                        }
                    }
                    Text { small: true, muted: true,
                        "Enter the seed of an earlier roll to get the same week again. While a seed is set, rolling again gives the same week every time."
                    }
                }
            }
        }
    }
}
