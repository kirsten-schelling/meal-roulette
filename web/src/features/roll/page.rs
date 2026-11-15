//! The roll page: choose how many dinners, add any rules, and roll. The roll is a
//! preview that shows in place; Keep stores it as a plan, so rolling again and
//! again leaves nothing behind in History.

use crate::components::ui::*;
use crate::features::cuisine::hooks::use_cuisine::use_get_cuisines;
use crate::features::ingredient::hooks::use_ingredient::use_get_ingredients;
use crate::features::protein::hooks::use_protein::use_get_proteins;
use crate::features::recipe::components::recipe_seed::RecipeSeed;
use crate::features::recipe::components::AddRecipeDialog;
use crate::features::roll::components::{
    ConflictAlert, FixButtons, PresetBar, RecipePickDialog, RollBar, RollErrorAlert, RollFailure,
    RollHero, RollResults, RolledWeek, RuleChoice, RulesPanel,
};
use crate::features::roll::fix::Fix;
use crate::features::roll::hooks::use_fit_count::use_fit_count;
use crate::features::roll::hooks::use_roll::{keep_roll, preview_roll, reroll_preview};
use crate::features::roll::settings::{
    band_label, EveryDinner, RollSettings, RuleTarget, DEFAULT_NUM_DAYS, MAX_ROLL_DAYS,
};
use crate::features::tag::hooks::use_tag::use_get_tags;
use crate::hooks::use_toast::use_toast;
use crate::router::Route;
use dioxus::prelude::*;
use domain::roll::{
    DifficultyBand, FitRequest, KeepRollRequest, PreviewMeal, RerollPreviewRequest,
    RollConstraints, RollRequest, RuleFamily,
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
    let tags = use_get_tags();
    let toast = use_toast();
    let navigator = use_navigator();

    let mut settings = use_signal(|| RollSettings {
        num_days: DEFAULT_NUM_DAYS.to_string(),
        ..RollSettings::default()
    });
    // An empty seed means random.
    let mut seed = use_signal(String::new);

    let mut rules_open = use_signal(|| false);
    let mut advanced_open = use_signal(|| false);

    let mut shown = use_signal(|| None as Option<Shown>);
    let mut roll_error = use_signal(|| None as Option<RollFailure>);
    let mut rolling = use_signal(|| false);
    let mut rerolling = use_signal(|| None as Option<usize>);
    let mut keeping = use_signal(|| false);
    // The add-recipe form opened from a conflict, filled in from its rule.
    let mut adding = use_signal(|| None as Option<RecipeSeed>);
    // The day being given a hand-picked recipe.
    let mut picking = use_signal(|| None as Option<usize>);

    let days = use_memo(move || settings.read().days());
    let fit_request = use_memo(move || {
        let days = days();
        (1..=MAX_ROLL_DAYS).contains(&days).then(|| FitRequest {
            num_days: days as u16,
            constraints: settings.read().constraints(),
        })
    });
    let mut fit = use_fit_count(fit_request);

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
        Some(_) if too_few => None,
        Some(c) => Some((
            format!(
                "{} of {} recipes fit your Must and every-dinner rules.",
                c.fitting, c.total
            ),
            false,
        )),
        None => None,
    };
    let shortage = fit_count.filter(|_| too_few).map(|c| {
        (
            format!(
                "Only {} recipe{} fit your Must and every-dinner rules, but you are rolling {}.",
                c.fitting,
                if c.fitting == 1 { "" } else { "s" },
                days()
            ),
            settings.read().shortage_fixes(c.fitting),
        )
    });
    let valid_days = (1..=MAX_ROLL_DAYS).contains(&days());
    let can_roll = valid_days && conflicts.is_empty() && !too_few;

    let mut choices: Vec<RuleChoice> = DifficultyBand::ALL
        .into_iter()
        .map(|band| RuleChoice {
            label: band_label(band).to_string(),
            target: RuleTarget::Band(band),
        })
        .collect();
    let mut load_failed = false;
    let mut add_lookups = |family: RuleFamily, rows: Option<Vec<(uuid::Uuid, String)>>| match rows {
        Some(rows) => choices.extend(rows.into_iter().map(|(id, name)| RuleChoice {
            label: name.clone(),
            target: RuleTarget::Lookup { family, id, name },
        })),
        None => load_failed = true,
    };
    add_lookups(
        RuleFamily::Protein,
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
        match ingredients() {
            Some(Ok(list)) => Some(list.into_iter().map(|i| (i.id, i.name)).collect()),
            _ => None,
        },
    );
    add_lookups(
        RuleFamily::Tag,
        match tags() {
            Some(Ok(list)) => Some(list.into_iter().map(|t| (t.tag.id, t.tag.name)).collect()),
            _ => None,
        },
    );
    let still_loading =
        proteins().is_none() || cuisines().is_none() || ingredients().is_none() || tags().is_none();
    let load_error = (load_failed && !still_loading).then(|| {
        "Some proteins, cuisines, ingredients or tags are missing from the list.".to_string()
    });

    let apply_fix = move |fix: Fix| {
        roll_error.set(None);
        match fix {
            Fix::MakeNiceToHave(rules) => settings.write().make_nice_to_have(&rules),
            Fix::ClearCaps => settings.write().every_dinner = EveryDinner::default(),
            Fix::RollDays(days) => settings.write().num_days = days.to_string(),
            Fix::AddRecipe(seed) => adding.set(Some(seed)),
        }
    };

    let do_roll = move |_| {
        if rolling() || keeping() {
            return;
        }
        let form = settings();
        let days = form.days();
        if !(1..=MAX_ROLL_DAYS).contains(&days) {
            return;
        }
        let constraints = form.constraints();
        let seed = seed().trim().parse::<i64>().ok();
        let catalog = ingredients().and_then(Result::ok).unwrap_or_default();
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
                            unmet: form.unmet_notes(&preview.unmet, &preview.meals, &catalog),
                            edited: false,
                            meals: preview.meals,
                        },
                        settings: form,
                        constraints,
                        recipe_ids,
                    }));
                }
                Err(e) => {
                    roll_error.set(Some(RollFailure::from(&e)));
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
        let catalog = ingredients().and_then(Result::ok).unwrap_or_default();
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
                    next.week.unmet =
                        next.settings
                            .unmet_notes(&replaced.unmet, &next.week.meals, &catalog);
                    next.week.edited = false;
                    shown.set(Some(next));
                }
                Err(e) => {
                    roll_error.set(Some(RollFailure::from(&e)));
                }
            }
            rerolling.set(None);
        });
    };

    let mut do_pick = move |(day, meal): (usize, PreviewMeal)| {
        picking.set(None);
        let Some(mut current) = shown() else {
            return;
        };
        if let Some(slot) = current.week.meals.get_mut(day) {
            *slot = meal;
        }
        current.recipe_ids = current.week.meals.iter().map(|m| m.recipe_id).collect();
        current.week.unmet.clear();
        current.week.edited = true;
        shown.set(Some(current));
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
                        "Week saved to Planned meals.".to_string(),
                        Tone::Green,
                    );
                    navigator.push(Route::PlannedMeals {});
                }
                Err(e) => {
                    roll_error.set(Some(RollFailure::from(&e)));
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
            if let Some((message, fixes)) = shortage {
                Alert { tone: Tone::Amber, title: "Not enough recipes for this week".to_string(),
                    p { class: "mr-alert__text", "{message}" }
                    FixButtons { fixes, on_fix: apply_fix }
                }
            }
            if let Some(failure) = roll_error() {
                RollErrorAlert {
                    message: failure.message,
                    fixes: if failure.rules_unmet { settings.read().failure_fixes() } else { Vec::new() },
                    on_fix: apply_fix,
                    on_dismiss: move |_| roll_error.set(None),
                }
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
                    on_pick: move |day| picking.set(Some(day)),
                    on_fix: apply_fix,
                }
            }

            if let Some(seed) = adding() {
                AddRecipeDialog {
                    seed,
                    on_close: move |_| adding.set(None),
                    on_saved: move |_| {
                        adding.set(None);
                        fit.restart();
                    },
                }
            }
            if let (Some(day), Some(current)) = (picking(), shown()) {
                RecipePickDialog {
                    day,
                    taken: current.recipe_ids,
                    on_pick: move |meal| do_pick((day, meal)),
                    on_close: move |_| picking.set(None),
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
                        "Use a past roll's seed to get the same week again."
                    }
                }
            }
        }
    }
}
