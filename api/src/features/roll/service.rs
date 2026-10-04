use std::collections::{HashMap, HashSet};

use domain::meal_plan::{MealPlanDetail, NewMealPlan};
use domain::planned_meal::{NewPlannedMeal, PlannedMealDetail};
use domain::recipe::Recipe;
use domain::roll::{CountRule, RollConstraints, RollRequest};
use solver::{CuisineRequirement, Problem, ProteinRequirement, SolveError};
use sqlx::PgPool;
use time::Duration;
use uuid::Uuid;

use crate::error::AppError;
use crate::features::{household, meal_plan, planned_meal, recipe};

/// Roll a new plan: pick `num_days` recipes under the request's constraints,
/// then store the plan and one dinner per day starting at `start_date`.
pub async fn roll_week(pool: &PgPool, request: RollRequest) -> Result<MealPlanDetail, AppError> {
    let num_days = usize::from(request.num_days);
    if num_days == 0 {
        return Err(AppError::BadRequest(
            "Pick at least one day to roll.".to_string(),
        ));
    }
    let household_id = match request.household_id {
        Some(id) => id,
        None => household::repository::default_household(pool).await?.id,
    };
    let recipes = recipe::repository::list(pool).await?;
    let seed = request.seed.unwrap_or_else(fresh_seed);
    let picks = solve(&recipes, &request.constraints, num_days, seed)?;
    let start_date = request
        .start_date
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());

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
    for (offset, &index) in picks.iter().enumerate() {
        planned_meal::repository::create(
            &mut *tx,
            NewPlannedMeal {
                meal_plan_id: plan.id,
                recipe_id: recipes[index].id,
                scheduled_date: start_date + Duration::days(offset as i64),
                is_pinned: false,
                target_difficulty: None,
            },
        )
        .await?;
    }
    tx.commit().await?;

    Ok(meal_plan::repository::get_detail(pool, plan.id).await?)
}

/// Replace one dinner with a fresh pick that keeps the rest of the plan valid:
/// the other days stay fixed, the candidate pool excludes every recipe already
/// on the plan, and only the part of each rule those days leave unmet is asked
/// of the new pick.
pub async fn reroll(pool: &PgPool, planned_meal_id: Uuid) -> Result<PlannedMealDetail, AppError> {
    let meal = planned_meal::repository::get_detail(pool, planned_meal_id).await?;
    let plan = meal_plan::repository::get(pool, meal.meal_plan_id).await?;
    let kept: Vec<PlannedMealDetail> = planned_meal::repository::list_detail(pool, plan.id)
        .await?
        .into_iter()
        .filter(|other| other.id != meal.id)
        .collect();

    let taken: HashSet<Uuid> = kept
        .iter()
        .map(|other| other.recipe_id)
        .chain(std::iter::once(meal.recipe_id))
        .collect();
    let candidates: Vec<Recipe> = recipe::repository::list(pool)
        .await?
        .into_iter()
        .filter(|candidate| !taken.contains(&candidate.id))
        .collect();

    let picks = solve(
        &candidates,
        &residual(&plan.constraints, &kept),
        1,
        fresh_seed(),
    )?;
    let replacement = candidates[picks[0]].id;
    Ok(planned_meal::repository::set_recipe(pool, meal.id, replacement).await?)
}

/// What the rules still demand once the kept days are counted: a rule for two
/// Thai nights with one Thai dinner already in place becomes a rule for one.
fn residual(constraints: &RollConstraints, kept: &[PlannedMealDetail]) -> RollConstraints {
    let remaining = |rules: &[CountRule], matches: fn(&PlannedMealDetail, Uuid) -> bool| {
        rules
            .iter()
            .filter_map(|rule| {
                let satisfied = kept.iter().filter(|meal| matches(meal, rule.id)).count();
                let times = rule
                    .times
                    .saturating_sub(u32::try_from(satisfied).unwrap_or(u32::MAX));
                (times > 0).then_some(CountRule { id: rule.id, times })
            })
            .collect()
    };
    RollConstraints {
        cuisines: remaining(&constraints.cuisines, |meal, id| {
            meal.cuisine_id == Some(id)
        }),
        proteins: remaining(&constraints.proteins, |meal, id| {
            meal.main_protein_id == Some(id)
        }),
    }
}

/// Flatten recipes and rules into the solver's integer-only input, run it, and
/// hand back positions into `recipes`.
fn solve(
    recipes: &[Recipe],
    constraints: &RollConstraints,
    num_to_pick: usize,
    seed: i64,
) -> Result<Vec<usize>, AppError> {
    let mut codes = Codes::default();
    let problem = Problem {
        recipes: recipes
            .iter()
            .map(|recipe| solver::Recipe {
                cuisine: codes.of(recipe.cuisine_id),
                protein: codes.of(recipe.main_protein_id),
            })
            .collect(),
        cuisine_requirements: constraints
            .cuisines
            .iter()
            .map(|rule| CuisineRequirement {
                cuisine: codes.of(Some(rule.id)),
                num_days: days(rule.times),
            })
            .collect(),
        protein_requirements: constraints
            .proteins
            .iter()
            .map(|rule| ProteinRequirement {
                protein: codes.of(Some(rule.id)),
                num_days: days(rule.times),
            })
            .collect(),
        num_to_pick,
        seed: seed as u64,
    };
    solver::roll(&problem).map_err(solver_error)
}

/// Hands out the small positive integers the solver works with, one per lookup
/// id. A recipe with no value gets 0, which the solver reads as "unspecified".
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

fn days(times: u32) -> i32 {
    i32::try_from(times).unwrap_or(i32::MAX)
}

/// Seeds stay small and positive so they are easy to read off the plan and
/// type back in to repeat a roll.
fn fresh_seed() -> i64 {
    i64::from(rand::random::<u32>())
}

fn solver_error(error: SolveError) -> AppError {
    match error {
        SolveError::Unsatisfiable => AppError::Unprocessable(
            "No combination of your recipes satisfies these constraints. Loosen a rule, roll fewer days, or add recipes."
                .to_string(),
        ),
        SolveError::InvalidInput => AppError::BadRequest(error.to_string()),
        SolveError::Unknown(_) | SolveError::BadIndex(_) | SolveError::Duplicate(_) => {
            AppError::Internal(error.to_string())
        }
    }
}
