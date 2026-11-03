use std::collections::{HashMap, HashSet};

use domain::meal_plan::NewMealPlan;
use domain::planned_meal::{NewPlannedMeal, PlannedMealDetail};
use domain::recipe::Recipe;
use domain::roll::{
    Bound, CountRule, DayLimitPlan, DifficultyRule, Importance, RollConstraints, RollOutcome,
    RollRequest, RuleFamily, UnmetRule,
};
use solver::{Problem, Requirement, SlotLimit, SolveError};
use sqlx::PgPool;
use time::Duration;
use uuid::Uuid;

use crate::error::AppError;
use crate::features::{household, meal_plan, planned_meal, recipe, recipe_ingredient};

/// The most days one roll may ask for. The solver's memory and recursion depth grow with
/// the number of days, and a plan longer than a month has no use.
const MAX_ROLL_DAYS: usize = 31;

fn check_num_days(num_days: usize) -> Result<usize, AppError> {
    if num_days == 0 {
        return Err(AppError::BadRequest(
            "Pick at least one day to roll.".to_string(),
        ));
    }
    if num_days > MAX_ROLL_DAYS {
        return Err(AppError::BadRequest(format!(
            "You can roll at most {MAX_ROLL_DAYS} days at a time."
        )));
    }
    Ok(num_days)
}

/// The solver is CPU-bound and can run for seconds on a tangled rule set, so it
/// runs on the blocking pool rather than on an async worker.
async fn run_solver(problem: Problem) -> Result<solver::Solution, AppError> {
    tokio::task::spawn_blocking(move || solver::roll(&problem))
        .await
        .map_err(|error| AppError::Internal(format!("the solver task failed: {error}")))?
        .map_err(solver_error)
}

/// Roll a new plan: pick `num_days` recipes under the request's constraints,
/// then store the plan and its dinners in roll order. With a `start_date` they are
/// also placed on consecutive dates from it; without one they are left unscheduled.
/// The outcome lists the preferences the recipes could not fully meet.
pub async fn roll_week(pool: &PgPool, request: RollRequest) -> Result<RollOutcome, AppError> {
    let num_days = check_num_days(usize::from(request.num_days))?;
    let household_id = match request.household_id {
        Some(id) => id,
        None => household::repository::default_household(pool).await?.id,
    };
    check_day_limits(request.constraints.day_limits.as_ref())?;
    let recipes = recipe::repository::list(pool).await?;
    let grid = Grid::build(pool, &recipes, &request.constraints).await?;
    let seed = request.seed.unwrap_or_else(fresh_seed);
    let start_date = request.start_date;
    let slot_limits = request
        .constraints
        .day_limits
        .as_ref()
        .map(|plan| {
            (0..num_days)
                .flat_map(|day| slot_limits_for(plan, day))
                .collect()
        })
        .unwrap_or_default();
    let solution = run_solver(Problem {
        recipes: grid.rows.clone(),
        requirements: grid.requirements(),
        slot_limits,
        num_to_pick: num_days,
        seed: seed as u64,
    })
    .await?;
    let unmet = grid.unmet(&solution.achieved);

    let mut tx = pool.begin().await?;
    let plan = meal_plan::repository::create(
        &mut *tx,
        NewMealPlan {
            household_id,
            start_date,
            seed: Some(seed),
            name: None,
            constraints: request.constraints,
        },
    )
    .await?;
    for (offset, &index) in solution.picks.iter().enumerate() {
        planned_meal::repository::create(
            &mut *tx,
            NewPlannedMeal {
                meal_plan_id: plan.id,
                recipe_id: recipes[index].id,
                day_index: i16::try_from(offset + 1)
                    .map_err(|_| AppError::Internal("too many days to number".to_string()))?,
                scheduled_date: start_date.map(|start| start + Duration::days(offset as i64)),
                is_pinned: false,
                target_difficulty: None,
            },
        )
        .await?;
    }
    tx.commit().await?;

    Ok(RollOutcome {
        plan: meal_plan::repository::get_detail(pool, plan.id).await?,
        unmet,
    })
}

/// Replace one dinner with a fresh pick that keeps the rest of the plan valid:
/// the other days stay fixed, the candidate pool excludes every recipe already
/// on the plan, and only the part of each rule those days leave open is asked
/// of the new pick.
pub async fn reroll(pool: &PgPool, planned_meal_id: Uuid) -> Result<PlannedMealDetail, AppError> {
    let meal = planned_meal::repository::get_detail(pool, planned_meal_id).await?;
    let plan = meal_plan::repository::get(pool, meal.meal_plan_id).await?;
    let kept: HashSet<Uuid> = planned_meal::repository::list_detail(pool, plan.id)
        .await?
        .into_iter()
        .filter(|other| other.id != meal.id)
        .map(|other| other.recipe_id)
        .collect();

    let recipes = recipe::repository::list(pool).await?;
    let grid = Grid::build(pool, &recipes, &plan.constraints).await?;
    let kept_rows: Vec<&Vec<i32>> = recipes
        .iter()
        .zip(&grid.rows)
        .filter(|(r, _)| kept.contains(&r.id))
        .map(|(_, row)| row)
        .collect();
    let candidates: Vec<usize> = recipes
        .iter()
        .enumerate()
        .filter(|(_, r)| !kept.contains(&r.id) && r.id != meal.recipe_id)
        .map(|(i, _)| i)
        .collect();

    let solution = run_solver(Problem {
        recipes: candidates.iter().map(|&i| grid.rows[i].clone()).collect(),
        requirements: residual(&grid.requirements(), &kept_rows),
        slot_limits: plan
            .constraints
            .day_limits
            .as_ref()
            .map(|limits| {
                let day = usize::try_from(meal.day_index - 1).unwrap_or(0);
                reroll_slot_limits(limits, day)
            })
            .unwrap_or_default(),
        num_to_pick: 1,
        seed: fresh_seed() as u64,
    })
    .await?;
    let replacement = recipes[candidates[solution.picks[0]]].id;
    Ok(planned_meal::repository::set_recipe(pool, meal.id, replacement).await?)
}

fn check_day_limits(plan: Option<&DayLimitPlan>) -> Result<(), AppError> {
    if plan.is_none_or(DayLimitPlan::is_valid) {
        return Ok(());
    }
    Err(AppError::BadRequest(
        "A day limit can't be met by any recipe: the easiest allowed difficulty is above the hardest, or the cook time is zero."
            .to_string(),
    ))
}

/// Limits for re-rolling plan day `day`: the solver is asked for a single pick,
/// so its one open slot is always slot 0.
fn reroll_slot_limits(plan: &DayLimitPlan, day: usize) -> Vec<SlotLimit> {
    slot_limits_for(plan, day)
        .into_iter()
        .map(|l| SlotLimit { day: 0, ..l })
        .collect()
}

/// The solver limits for roll day `day` (0 is the first day).
fn slot_limits_for(plan: &DayLimitPlan, day: usize) -> Vec<SlotLimit> {
    let limits = plan.limits_for(day);
    let (low, high) = limits.levels();
    let mut slot_limits = Vec::new();
    if (low, high) != (1, 5) {
        slot_limits.push(SlotLimit {
            day,
            attribute: DIFFICULTY,
            low: i32::from(low),
            high: i32::from(high),
        });
    }
    if let Some(max) = limits.max_time_mins {
        slot_limits.push(SlotLimit {
            day,
            attribute: TIME,
            low: 0,
            high: i32::try_from(max).unwrap_or(i32::MAX),
        });
    }
    slot_limits
}

/// What the rules still ask of the open day once the kept days are counted: at
/// least two Thai nights with one Thai dinner kept becomes at least one; at
/// most two chicken with two kept becomes none.
fn residual(requirements: &[Requirement], kept: &[&Vec<i32>]) -> Vec<Requirement> {
    requirements
        .iter()
        .map(|r| {
            let matched = kept.iter().filter(|row| r.matches(row)).count();
            let matched = u32::try_from(matched).unwrap_or(u32::MAX);
            Requirement {
                at_least: r.at_least.saturating_sub(matched),
                at_most: r.at_most.map(|max| max.saturating_sub(matched)),
                ..*r
            }
        })
        .collect()
}

// The solver's columns. Must-use ingredients take one 0/1 column each, after these.
const CUISINE: usize = 0;
const PROTEIN: usize = 1;
const DIFFICULTY: usize = 2;
const TIME: usize = 3;
const FIRST_INGREDIENT: usize = 4;

/// The recipes flattened into the solver's integer grid, plus each rule as a
/// requirement over it, remembering which rule each requirement came from.
struct Grid {
    rows: Vec<Vec<i32>>,
    rules: Vec<(RuleFamily, usize, Requirement)>,
}

impl Grid {
    async fn build(
        pool: &PgPool,
        recipes: &[Recipe],
        constraints: &RollConstraints,
    ) -> Result<Grid, AppError> {
        let ingredient_ids: Vec<Uuid> = constraints.ingredients.iter().map(|r| r.id).collect();
        let uses: HashSet<(Uuid, Uuid)> =
            recipe_ingredient::repository::uses_of(pool, &ingredient_ids)
                .await?
                .into_iter()
                .map(|u| (u.recipe_id, u.ingredient_id))
                .collect();
        Ok(Grid::new(recipes, &uses, constraints))
    }

    /// `uses` holds (recipe id, ingredient id) for every recipe using a must-use ingredient.
    fn new(
        recipes: &[Recipe],
        uses: &HashSet<(Uuid, Uuid)>,
        constraints: &RollConstraints,
    ) -> Grid {
        let ingredient_ids: Vec<Uuid> = constraints.ingredients.iter().map(|r| r.id).collect();
        let mut codes = Codes::default();
        let rows = recipes
            .iter()
            .map(|r| {
                let mut row = vec![
                    codes.of(r.cuisine_id),
                    codes.of(r.main_protein_id),
                    i32::from(r.difficulty),
                    i32::from(r.time_mins),
                ];
                row.extend(
                    ingredient_ids
                        .iter()
                        .map(|&ingredient| i32::from(uses.contains(&(r.id, ingredient)))),
                );
                row
            })
            .collect();

        let mut rules = Vec::new();
        let mut lookup = |family, attribute, list: &[CountRule], codes: &mut Codes| {
            for (index, rule) in list.iter().enumerate() {
                let value = codes.of(Some(rule.id));
                let requirement = counted(
                    attribute,
                    value,
                    value,
                    rule.times,
                    rule.bound,
                    rule.importance,
                );
                rules.push((family, index, requirement));
            }
        };
        lookup(
            RuleFamily::Cuisine,
            CUISINE,
            &constraints.cuisines,
            &mut codes,
        );
        lookup(
            RuleFamily::Protein,
            PROTEIN,
            &constraints.proteins,
            &mut codes,
        );
        for (index, rule) in constraints.ingredients.iter().enumerate() {
            let requirement = counted(
                FIRST_INGREDIENT + index,
                1,
                1,
                rule.times,
                rule.bound,
                rule.importance,
            );
            rules.push((RuleFamily::Ingredient, index, requirement));
        }
        for (index, rule) in constraints.difficulties.iter().enumerate() {
            rules.push((RuleFamily::Difficulty, index, difficulty(rule)));
        }
        Grid { rows, rules }
    }

    fn requirements(&self) -> Vec<Requirement> {
        self.rules.iter().map(|(_, _, r)| *r).collect()
    }

    /// The rules the week falls short of, given how many picks matched each.
    fn unmet(&self, achieved: &[u32]) -> Vec<UnmetRule> {
        self.rules
            .iter()
            .zip(achieved)
            .filter(|((_, _, r), &got)| !r.is_met(got))
            .map(|(&(family, index, r), &got)| {
                let (bound, times) = match r.at_most {
                    Some(max) if r.at_least == 0 => (Bound::AtMost, max),
                    _ => (Bound::AtLeast, r.at_least),
                };
                UnmetRule {
                    family,
                    index,
                    bound,
                    times,
                    got,
                }
            })
            .collect()
    }
}

fn counted(
    attribute: usize,
    low: i32,
    high: i32,
    times: u32,
    bound: Bound,
    importance: Importance,
) -> Requirement {
    let (at_least, at_most) = match bound {
        Bound::AtLeast => (times, None),
        Bound::AtMost => (0, Some(times)),
    };
    Requirement {
        attribute,
        low,
        high,
        at_least,
        at_most,
        importance: match importance {
            Importance::Low => solver::Importance::Low,
            Importance::Medium => solver::Importance::Medium,
            Importance::High => solver::Importance::High,
            Importance::Required => solver::Importance::Required,
        },
    }
}

fn difficulty(rule: &DifficultyRule) -> Requirement {
    let (low, high) = rule.band.levels();
    counted(
        DIFFICULTY,
        i32::from(low),
        i32::from(high),
        rule.times,
        rule.bound,
        rule.importance,
    )
}

/// Hands out the small positive integers the solver works with, one per lookup
/// id. A recipe with no value gets 0, which no rule asks for.
#[derive(Default)]
struct Codes(HashMap<Uuid, i32>);

impl Codes {
    fn of(&mut self, id: Option<Uuid>) -> i32 {
        let Some(id) = id else {
            return 0;
        };
        let next = i32::try_from(self.0.len() + 1).unwrap_or(i32::MAX);
        *self.0.entry(id).or_insert(next)
    }
}

/// Seeds stay small and positive so they are easy to read off the plan and
/// type back in to repeat a roll.
fn fresh_seed() -> i64 {
    i64::from(rand::random::<u32>())
}

fn solver_error(error: SolveError) -> AppError {
    match error {
        SolveError::Unsatisfiable => AppError::Unprocessable(
            "No combination of your recipes meets the required rules and day limits. Loosen a rule or a day limit, make a rule a preference, roll fewer days, or add recipes."
                .to_string(),
        ),
        SolveError::GaveUp => AppError::Unprocessable(
            "These required rules are too tangled to settle quickly. Make one of them a preference, or loosen it."
                .to_string(),
        ),
        SolveError::InvalidInput => AppError::BadRequest(error.to_string()),
        SolveError::NoMemory
        | SolveError::Unknown(_)
        | SolveError::BadIndex(_)
        | SolveError::BrokenLimit(_)
        | SolveError::Duplicate(_) => {
            AppError::Internal(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::roll::{DayLimits, DayLimitsAt, DifficultyBand};

    fn recipe(cuisine: Uuid, protein: Uuid, difficulty: i16) -> Recipe {
        Recipe {
            id: Uuid::now_v7(),
            household_id: None,
            title: String::new(),
            difficulty,
            time_mins: 20,
            servings: 2,
            healthiness: None,
            main_protein_id: Some(protein),
            cuisine_id: Some(cuisine),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
        }
    }

    fn rule(id: Uuid, times: u32, bound: Bound, importance: Importance) -> CountRule {
        CountRule {
            id,
            times,
            bound,
            importance,
        }
    }

    fn solve(grid: &Grid, days: usize, seed: u64) -> Result<solver::Solution, SolveError> {
        solver::roll(&Problem {
            recipes: grid.rows.clone(),
            requirements: grid.requirements(),
            slot_limits: Vec::new(),
            num_to_pick: days,
            seed,
        })
    }

    #[test]
    fn num_days_must_be_between_one_and_the_cap() {
        assert!(matches!(check_num_days(0), Err(AppError::BadRequest(_))));
        assert_eq!(check_num_days(1).ok(), Some(1));
        assert_eq!(check_num_days(MAX_ROLL_DAYS).ok(), Some(MAX_ROLL_DAYS));
        assert!(matches!(
            check_num_days(MAX_ROLL_DAYS + 1),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn every_rule_family_reaches_the_solver() {
        let (thai, italian, chicken, beef, tofu, spinach) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let recipes = vec![
            recipe(thai, chicken, 1),
            recipe(thai, beef, 2),
            recipe(thai, tofu, 5),
            recipe(italian, chicken, 4),
            recipe(italian, beef, 3),
            recipe(italian, tofu, 1),
            recipe(thai, chicken, 3),
            recipe(italian, beef, 5),
        ];
        let uses: HashSet<(Uuid, Uuid)> = [(recipes[5].id, spinach), (recipes[1].id, spinach)]
            .into_iter()
            .collect();
        let constraints = RollConstraints {
            cuisines: vec![rule(thai, 2, Bound::AtLeast, Importance::Required)],
            proteins: vec![rule(chicken, 1, Bound::AtMost, Importance::Required)],
            ingredients: vec![rule(spinach, 1, Bound::AtLeast, Importance::Required)],
            difficulties: vec![DifficultyRule {
                band: DifficultyBand::Hard,
                times: 1,
                bound: Bound::AtLeast,
                importance: Importance::Required,
            }],
            day_limits: None,
        };
        let grid = Grid::new(&recipes, &uses, &constraints);
        for seed in 0..40 {
            let solution = solve(&grid, 4, seed).expect("the rules fit together");
            let picked: Vec<&Recipe> = solution.picks.iter().map(|&i| &recipes[i]).collect();
            let count = |test: &dyn Fn(&Recipe) -> bool| picked.iter().filter(|r| test(r)).count();
            assert!(count(&|r| r.cuisine_id == Some(thai)) >= 2, "seed {seed}");
            assert!(
                count(&|r| r.main_protein_id == Some(chicken)) <= 1,
                "seed {seed}"
            );
            assert!(
                count(&|r| uses.contains(&(r.id, spinach))) >= 1,
                "seed {seed}"
            );
            assert!(count(&|r| r.difficulty >= 4) >= 1, "seed {seed}");
            assert!(grid.unmet(&solution.achieved).is_empty(), "seed {seed}");
        }
    }

    #[test]
    fn a_loosened_preference_is_reported_against_its_rule() {
        let (thai, italian, chicken) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
        let recipes = vec![
            recipe(thai, chicken, 2),
            recipe(thai, chicken, 3),
            recipe(italian, chicken, 2),
            recipe(italian, chicken, 4),
        ];
        let constraints = RollConstraints {
            proteins: vec![rule(chicken, 0, Bound::AtMost, Importance::Low)],
            cuisines: vec![rule(thai, 3, Bound::AtLeast, Importance::High)],
            ..RollConstraints::default()
        };
        let grid = Grid::new(&recipes, &HashSet::new(), &constraints);
        let solution = solve(&grid, 3, 1).expect("preferences never fail a roll");
        let unmet = grid.unmet(&solution.achieved);
        assert!(unmet.contains(&UnmetRule {
            family: RuleFamily::Cuisine,
            index: 0,
            bound: Bound::AtLeast,
            times: 3,
            got: 2,
        }));
        assert!(unmet.contains(&UnmetRule {
            family: RuleFamily::Protein,
            index: 0,
            bound: Bound::AtMost,
            times: 0,
            got: 3,
        }));
    }

    #[test]
    fn a_required_rule_for_a_value_no_recipe_has_cannot_be_met() {
        let (thai, chicken, korean) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
        let recipes = vec![recipe(thai, chicken, 2), recipe(thai, chicken, 3)];
        let constraints = RollConstraints {
            cuisines: vec![rule(korean, 1, Bound::AtLeast, Importance::Required)],
            ..RollConstraints::default()
        };
        let grid = Grid::new(&recipes, &HashSet::new(), &constraints);
        assert_eq!(solve(&grid, 1, 0), Err(SolveError::Unsatisfiable));
    }

    #[test]
    fn the_residual_counts_the_kept_days() {
        let at_least_two_thai = counted(CUISINE, 1, 1, 2, Bound::AtLeast, Importance::Required);
        let at_most_one_chicken = counted(PROTEIN, 7, 7, 1, Bound::AtMost, Importance::High);
        let kept = [vec![1, 7, 3], vec![2, 8, 3]];
        let kept: Vec<&Vec<i32>> = kept.iter().collect();
        let left = residual(&[at_least_two_thai, at_most_one_chicken], &kept);
        assert_eq!((left[0].at_least, left[0].at_most), (1, None));
        assert_eq!((left[1].at_least, left[1].at_most), (0, Some(0)));
        assert_eq!(left[1].importance, solver::Importance::High);
    }

    fn timed(difficulty: i16, time_mins: i16) -> Recipe {
        Recipe {
            time_mins,
            ..recipe(Uuid::now_v7(), Uuid::now_v7(), difficulty)
        }
    }

    fn limit_plan() -> DayLimitPlan {
        DayLimitPlan {
            default: DayLimits {
                max_difficulty: Some(DifficultyBand::Medium),
                max_time_mins: Some(30),
                ..DayLimits::default()
            },
            overrides: [5, 6]
                .into_iter()
                .map(|day| DayLimitsAt {
                    day,
                    limits: DayLimits {
                        min_difficulty: Some(DifficultyBand::Medium),
                        ..DayLimits::default()
                    },
                })
                .collect(),
        }
    }

    #[test]
    fn a_roll_gives_each_day_number_its_own_limits() {
        let recipes: Vec<Recipe> = (0..6)
            .map(|n| timed(1 + n % 3, 15 + 3 * n))
            .chain((0..6).map(|n| timed(3 + n % 3, 45 + 10 * n)))
            .collect();
        let grid = Grid::new(&recipes, &HashSet::new(), &RollConstraints::default());
        let plan = limit_plan();
        for seed in 0..40 {
            let solution = solver::roll(&Problem {
                recipes: grid.rows.clone(),
                requirements: grid.requirements(),
                slot_limits: (0..7).flat_map(|d| slot_limits_for(&plan, d)).collect(),
                num_to_pick: 7,
                seed,
            })
            .expect("quick and ambitious recipes both exist");
            for (day, &pick) in solution.picks.iter().enumerate() {
                let limits = plan.limits_for(day);
                let r = &recipes[pick];
                let (low, high) = limits.levels();
                assert!(
                    (low..=high).contains(&r.difficulty),
                    "seed {seed} day {day}"
                );
                assert!(
                    limits
                        .max_time_mins
                        .is_none_or(|max| r.time_mins <= max as i16),
                    "seed {seed} day {day}"
                );
            }
        }
    }

    #[test]
    fn rerolling_a_later_day_applies_that_days_limits_to_the_single_slot() {
        let recipes: Vec<Recipe> = (0..6)
            .map(|n| timed(1 + n % 3, 15 + 3 * n))
            .chain((0..6).map(|n| timed(3 + n % 3, 45 + 10 * n)))
            .collect();
        let grid = Grid::new(&recipes, &HashSet::new(), &RollConstraints::default());
        let plan = limit_plan();
        for seed in 0..20 {
            let solution = solver::roll(&Problem {
                recipes: grid.rows.clone(),
                requirements: grid.requirements(),
                slot_limits: reroll_slot_limits(&plan, 5),
                num_to_pick: 1,
                seed,
            })
            .expect("a medium or harder recipe exists");
            assert!(recipes[solution.picks[0]].difficulty >= 3, "seed {seed}");
        }
    }

    #[test]
    fn an_unlimited_override_adds_no_solver_limits() {
        let plan = DayLimitPlan {
            overrides: vec![DayLimitsAt {
                day: 0,
                limits: DayLimits::default(),
            }],
            ..limit_plan()
        };
        assert!(slot_limits_for(&plan, 0).is_empty());
        let limits = slot_limits_for(&plan, 1);
        assert_eq!(limits.len(), 2);
        assert!(limits.iter().all(|l| l.day == 1));
    }

    #[test]
    fn days_with_too_few_quick_recipes_cannot_be_rolled() {
        let recipes: Vec<Recipe> = (0..8)
            .map(|n| timed(1, if n < 2 { 20 } else { 90 }))
            .collect();
        let grid = Grid::new(&recipes, &HashSet::new(), &RollConstraints::default());
        let plan = DayLimitPlan {
            default: DayLimits {
                max_time_mins: Some(30),
                ..DayLimits::default()
            },
            overrides: Vec::new(),
        };
        let result = solver::roll(&Problem {
            recipes: grid.rows.clone(),
            requirements: grid.requirements(),
            slot_limits: (0..5).flat_map(|d| slot_limits_for(&plan, d)).collect(),
            num_to_pick: 5,
            seed: 1,
        });
        assert_eq!(result, Err(SolveError::Unsatisfiable));
    }

    #[test]
    fn day_limits_no_recipe_could_meet_are_a_bad_request() {
        let backwards = DayLimitPlan {
            default: DayLimits::default(),
            overrides: vec![DayLimitsAt {
                day: 3,
                limits: DayLimits {
                    min_difficulty: Some(DifficultyBand::Hard),
                    max_difficulty: Some(DifficultyBand::Easy),
                    max_time_mins: None,
                },
            }],
        };
        assert!(matches!(
            check_day_limits(Some(&backwards)),
            Err(AppError::BadRequest(_))
        ));
        assert!(check_day_limits(Some(&limit_plan())).is_ok());
        assert!(check_day_limits(None).is_ok());
    }
}
