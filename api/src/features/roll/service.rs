use std::collections::{HashMap, HashSet};

use domain::meal_plan::{MealPlanDetail, NewMealPlan};
use domain::planned_meal::{NewPlannedMeal, PlannedMealDetail};
use domain::recipe::{Recipe, RecipeOverview};
use domain::roll::{
    Bound, CountRule, DayLimitPlan, DifficultyRule, FitCount, FitRequest, Importance,
    KeepRollRequest, PinnedDinner, PreviewMeal, RerollPreview, RerollPreviewRequest,
    RollConstraints, RollOutcome, RollPreview, RollRequest, RuleFamily, UnmetRule,
};
use solver::{Problem, Requirement, SlotLimit, SolveError};
use sqlx::PgPool;
use time::{Date, Duration, OffsetDateTime};
use uuid::Uuid;

use crate::error::AppError;
use crate::features::{
    cook_log, household, meal_plan, planned_meal, recipe, recipe_ingredient, recipe_tag,
};

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

/// A solved roll that is not stored yet.
struct Rolled {
    recipes: Vec<Recipe>,
    grid: Grid,
    seed: i64,
    picks: Vec<usize>,
}

impl Rolled {
    fn recipe_ids(&self) -> Vec<Uuid> {
        self.picks.iter().map(|&i| self.recipes[i].id).collect()
    }

    fn unmet(&self) -> Vec<UnmetRule> {
        self.grid.unmet(&self.grid.achieved(&self.picks))
    }
}

async fn solve(pool: &PgPool, request: &RollRequest) -> Result<Rolled, AppError> {
    let num_days = check_num_days(usize::from(request.num_days))?;
    check_day_limits(request.constraints.day_limits.as_ref())?;
    let recipes = recipe::repository::list(pool).await?;
    let grid = Grid::build(pool, &recipes, &request.constraints).await?;
    let seed = request.seed.unwrap_or_else(fresh_seed);
    let pinned = pinned_days(&recipes, &request.pinned, num_days)?;
    let open_days: Vec<usize> = (0..num_days)
        .filter(|day| !pinned.contains_key(day))
        .collect();
    let pinned_rows: Vec<&Vec<i32>> = pinned.values().map(|&i| &grid.rows[i]).collect();
    if pins_exceed_a_required_maximum(&grid.requirements(), &pinned_rows) {
        return Err(suggest_unpinning(
            solver_error(SolveError::Unsatisfiable),
            &request.pinned,
        ));
    }
    let kept: HashSet<Uuid> = pinned.values().map(|&i| recipes[i].id).collect();
    let mut open = pick_open_days(
        &grid,
        &recipes,
        &kept,
        None,
        request.constraints.day_limits.as_ref(),
        &open_days,
        seed as u64,
    )
    .await
    .map_err(|error| suggest_unpinning(error, &request.pinned))?
    .into_iter();
    let picks = (0..num_days)
        .filter_map(|day| pinned.get(&day).copied().or_else(|| open.next()))
        .collect();
    Ok(Rolled {
        recipes,
        grid,
        seed,
        picks,
    })
}

/// A dinner in the cooking log keeps its recipe, so the log stays a record of
/// what was cooked.
async fn refuse_if_cooked(pool: &PgPool, planned_meal_id: Uuid) -> Result<(), AppError> {
    if cook_log::repository::find_for_planned_meal(pool, planned_meal_id)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(
            "This dinner is in the cooking log. Undo it on Home or remove it from the Cooking Log first, then swap or re-roll it."
                .to_string(),
        ));
    }
    Ok(())
}

/// Whether the pinned dinners alone already match a required at-most or exactly
/// rule more often than it allows, which no choice for the open days can undo.
fn pins_exceed_a_required_maximum(requirements: &[Requirement], pinned: &[&Vec<i32>]) -> bool {
    requirements
        .iter()
        .filter(|r| r.importance == solver::Importance::Required)
        .any(|r| {
            let matched = pinned.iter().filter(|row| r.matches(row)).count();
            r.at_most
                .is_some_and(|max| u32::try_from(matched).unwrap_or(u32::MAX) > max)
        })
}

/// The pinned dinners as day to index into `recipes`, each checked against the roll.
fn pinned_days(
    recipes: &[Recipe],
    pinned: &[PinnedDinner],
    num_days: usize,
) -> Result<HashMap<usize, usize>, AppError> {
    let position: HashMap<Uuid, usize> =
        recipes.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
    let mut days = HashMap::new();
    for pin in pinned {
        let day = usize::from(pin.day);
        if day >= num_days {
            return Err(AppError::BadRequest(
                "A pinned dinner is on a day past the end of the roll.".to_string(),
            ));
        }
        let Some(&index) = position.get(&pin.recipe_id) else {
            return Err(AppError::BadRequest(
                "A pinned recipe no longer exists. Unpin it and roll again.".to_string(),
            ));
        };
        if days.values().any(|&taken| taken == index) {
            return Err(AppError::BadRequest(
                "A recipe can only be in a roll once.".to_string(),
            ));
        }
        if days.insert(day, index).is_some() {
            return Err(AppError::BadRequest(
                "Two dinners are pinned to the same day.".to_string(),
            ));
        }
    }
    Ok(days)
}

async fn household_or_default(pool: &PgPool, id: Option<Uuid>) -> Result<Uuid, AppError> {
    match id {
        Some(id) => Ok(id),
        None => Ok(household::repository::default_household(pool).await?.id),
    }
}

/// Store a plan with its dinners in roll order. With a `start_date` they are
/// also placed on consecutive dates from it; without one they are left unscheduled.
async fn store_plan(
    pool: &PgPool,
    new_plan: NewMealPlan,
    recipe_ids: &[Uuid],
) -> Result<MealPlanDetail, AppError> {
    let start_date = new_plan.start_date;
    let mut tx = pool.begin().await?;
    let plan = meal_plan::repository::create(&mut *tx, new_plan).await?;
    for (offset, &recipe_id) in recipe_ids.iter().enumerate() {
        planned_meal::repository::create(
            &mut *tx,
            NewPlannedMeal {
                meal_plan_id: plan.id,
                recipe_id,
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
    Ok(meal_plan::repository::get_detail(pool, plan.id).await?)
}

/// Roll a new plan: pick `num_days` recipes under the request's constraints,
/// then store the plan and its dinners in roll order. With a `start_date` they are
/// also placed on consecutive dates from it; without one they are left unscheduled.
/// The outcome lists the preferences the recipes could not fully meet.
pub async fn roll_week(pool: &PgPool, request: RollRequest) -> Result<RollOutcome, AppError> {
    let rolled = solve(pool, &request).await?;
    let unmet = rolled.unmet();
    let household_id = household_or_default(pool, request.household_id).await?;
    let plan = store_plan(
        pool,
        NewMealPlan {
            household_id,
            start_date: request.start_date,
            seed: Some(rolled.seed),
            name: None,
            constraints: request.constraints,
        },
        &rolled.recipe_ids(),
    )
    .await?;
    Ok(RollOutcome { plan, unmet })
}

async fn overviews_by_id(pool: &PgPool) -> Result<HashMap<Uuid, RecipeOverview>, AppError> {
    Ok(recipe::repository::list_overview(pool)
        .await?
        .into_iter()
        .map(|overview| (overview.recipe.id, overview))
        .collect())
}

fn preview_meal(overview: &RecipeOverview, within_min_gap: bool) -> PreviewMeal {
    PreviewMeal {
        recipe_id: overview.recipe.id,
        title: overview.recipe.title.clone(),
        difficulty: overview.recipe.difficulty,
        time_mins: overview.recipe.time_mins,
        cuisine: overview.cuisine.clone(),
        protein: overview.protein.clone(),
        within_min_gap,
    }
}

/// Roll without storing anything. The result carries the seed and the recipes,
/// which `keep_roll` stores as they are.
pub async fn preview_roll(pool: &PgPool, request: RollRequest) -> Result<RollPreview, AppError> {
    let rolled = solve(pool, &request).await?;
    let overviews = overviews_by_id(pool).await?;
    let meals = rolled
        .picks
        .iter()
        .filter_map(|&i| {
            overviews
                .get(&rolled.recipes[i].id)
                .map(|overview| preview_meal(overview, rolled.grid.within_min_gap(i)))
        })
        .collect();
    Ok(RollPreview {
        seed: rolled.seed,
        meals,
        unmet: rolled.unmet(),
    })
}

/// Store a previewed roll as a plan, unscheduled and in roll order.
pub async fn keep_roll(
    pool: &PgPool,
    request: KeepRollRequest,
) -> Result<MealPlanDetail, AppError> {
    check_num_days(request.recipe_ids.len())?;
    let known: HashSet<Uuid> = recipe::repository::list(pool)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect();
    let distinct: HashSet<&Uuid> = request.recipe_ids.iter().collect();
    if distinct.len() != request.recipe_ids.len() {
        return Err(AppError::BadRequest(
            "A recipe can only be in a roll once.".to_string(),
        ));
    }
    if request.recipe_ids.iter().any(|id| !known.contains(id)) {
        return Err(AppError::BadRequest(
            "A recipe in this roll no longer exists. Roll again.".to_string(),
        ));
    }
    let household_id = household_or_default(pool, request.household_id).await?;
    store_plan(
        pool,
        NewMealPlan {
            household_id,
            start_date: None,
            seed: request.seed,
            name: None,
            constraints: request.constraints,
        },
        &request.recipe_ids,
    )
    .await
}

/// A recipe for each of `open_days` while the `kept` recipes stay on the other
/// days, as indexes into `recipes` in the order of `open_days`. `replaced` is
/// left out of the pool too, so a re-rolled day gets a different dinner.
async fn pick_open_days(
    grid: &Grid,
    recipes: &[Recipe],
    kept: &HashSet<Uuid>,
    replaced: Option<Uuid>,
    day_limits: Option<&DayLimitPlan>,
    open_days: &[usize],
    seed: u64,
) -> Result<Vec<usize>, AppError> {
    if open_days.is_empty() {
        return Ok(Vec::new());
    }
    let (problem, candidates) =
        open_problem(grid, recipes, kept, replaced, day_limits, open_days, seed);
    let solution = run_solver(problem).await?;
    Ok(solution
        .picks
        .iter()
        .map(|&pick| candidates[pick])
        .collect())
}

/// The solver's problem for the open days, and each of its rows' index into
/// `recipes`. The pool leaves out every kept recipe, only the part of each rule
/// the kept recipes leave open is asked of the picks, and the solver's slots
/// are the open days in order. With nothing kept or replaced it is the plain
/// roll, so a seed repeats the same week.
fn open_problem(
    grid: &Grid,
    recipes: &[Recipe],
    kept: &HashSet<Uuid>,
    replaced: Option<Uuid>,
    day_limits: Option<&DayLimitPlan>,
    open_days: &[usize],
    seed: u64,
) -> (Problem, Vec<usize>) {
    let kept_rows: Vec<&Vec<i32>> = recipes
        .iter()
        .zip(&grid.rows)
        .filter(|(r, _)| kept.contains(&r.id))
        .map(|(_, row)| row)
        .collect();
    let candidates: Vec<usize> = recipes
        .iter()
        .enumerate()
        .filter(|(_, r)| !kept.contains(&r.id) && Some(r.id) != replaced)
        .map(|(i, _)| i)
        .collect();
    let problem = Problem {
        recipes: candidates.iter().map(|&i| grid.rows[i].clone()).collect(),
        requirements: residual(&grid.requirements(), &kept_rows),
        slot_limits: day_limits
            .map(|plan| open_slot_limits(plan, open_days))
            .unwrap_or_default(),
        num_to_pick: open_days.len(),
        seed,
    };
    (problem, candidates)
}

/// Replace one dinner with a fresh pick that keeps the rest of the plan valid:
/// the other days stay fixed, the candidate pool excludes every recipe already
/// on the plan, and only the part of each rule those days leave open is asked
/// of the new pick.
pub async fn reroll(pool: &PgPool, planned_meal_id: Uuid) -> Result<PlannedMealDetail, AppError> {
    refuse_if_cooked(pool, planned_meal_id).await?;
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
    let day = usize::try_from(meal.day_index - 1).unwrap_or(0);
    let replacement = pick_open_days(
        &grid,
        &recipes,
        &kept,
        Some(meal.recipe_id),
        plan.constraints.day_limits.as_ref(),
        &[day],
        fresh_seed() as u64,
    )
    .await?[0];
    Ok(planned_meal::repository::set_recipe(pool, meal.id, recipes[replacement].id).await?)
}

/// Put a recipe picked by hand on a dinner that has not been cooked.
pub async fn swap(
    pool: &PgPool,
    planned_meal_id: Uuid,
    recipe_id: Uuid,
) -> Result<PlannedMealDetail, AppError> {
    refuse_if_cooked(pool, planned_meal_id).await?;
    Ok(planned_meal::repository::set_recipe(pool, planned_meal_id, recipe_id).await?)
}

/// Re-roll one dinner of a roll that is not stored.
pub async fn reroll_preview(
    pool: &PgPool,
    request: RerollPreviewRequest,
) -> Result<RerollPreview, AppError> {
    let day = usize::from(request.day);
    let Some(&replaced) = request.recipe_ids.get(day) else {
        return Err(AppError::BadRequest(
            "That day is not part of this roll.".to_string(),
        ));
    };
    let kept: HashSet<Uuid> = request
        .recipe_ids
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != day)
        .map(|(_, &id)| id)
        .collect();
    let recipes = recipe::repository::list(pool).await?;
    let grid = Grid::build(pool, &recipes, &request.constraints).await?;
    let replacement = pick_open_days(
        &grid,
        &recipes,
        &kept,
        Some(replaced),
        request.constraints.day_limits.as_ref(),
        &[day],
        fresh_seed() as u64,
    )
    .await?[0];

    let position: HashMap<Uuid, usize> =
        recipes.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
    let picks: Vec<usize> = request
        .recipe_ids
        .iter()
        .enumerate()
        .filter_map(|(i, id)| {
            if i == day {
                Some(replacement)
            } else {
                position.get(id).copied()
            }
        })
        .collect();
    let overviews = overviews_by_id(pool).await?;
    let meal = overviews
        .get(&recipes[replacement].id)
        .map(|overview| preview_meal(overview, grid.within_min_gap(replacement)))
        .ok_or_else(|| AppError::Internal("the new recipe disappeared".to_string()))?;
    Ok(RerollPreview {
        meal,
        unmet: grid.unmet(&grid.achieved(&picks)),
    })
}

/// How many recipes the roll can draw on: those no required "at most 0" rule
/// rules out and that some day of the roll accepts.
pub async fn fit_count(pool: &PgPool, request: FitRequest) -> Result<FitCount, AppError> {
    let recipes = recipe::repository::list(pool).await?;
    let grid = Grid::build(pool, &recipes, &request.constraints).await?;
    let fitting = fitting_rows(
        &grid.rows,
        &grid.requirements(),
        request.constraints.day_limits.as_ref(),
        usize::from(request.num_days).max(1),
    );
    Ok(FitCount {
        fitting: u32::try_from(fitting).unwrap_or(u32::MAX),
        total: u32::try_from(recipes.len()).unwrap_or(u32::MAX),
    })
}

fn fitting_rows(
    rows: &[Vec<i32>],
    requirements: &[Requirement],
    day_limits: Option<&DayLimitPlan>,
    num_days: usize,
) -> usize {
    let banned: Vec<&Requirement> = requirements
        .iter()
        .filter(|r| r.importance == solver::Importance::Required && r.at_most == Some(0))
        .collect();
    rows.iter()
        .filter(|row| !banned.iter().any(|r| r.matches(row)))
        .filter(|row| {
            day_limits.is_none_or(|plan| {
                (0..num_days).any(|day| {
                    let limits = plan.limits_for(day);
                    let (low, high) = limits.levels();
                    (i32::from(low)..=i32::from(high)).contains(&row[DIFFICULTY])
                        && limits
                            .max_time_mins
                            .is_none_or(|max| row[TIME] <= i32::try_from(max).unwrap_or(i32::MAX))
                })
            })
        })
        .count()
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

/// The limits of each open day, on the solver slot that day fills: the solver
/// numbers its slots 0.. in the order of `open_days`.
fn open_slot_limits(plan: &DayLimitPlan, open_days: &[usize]) -> Vec<SlotLimit> {
    open_days
        .iter()
        .enumerate()
        .flat_map(|(slot, &day)| {
            slot_limits_for(plan, day)
                .into_iter()
                .map(move |l| SlotLimit { day: slot, ..l })
        })
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

// The solver's columns. Must-use ingredients take one 0/1 column each, after
// these, and tag rules one each after the ingredients.
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
    /// The cook-gap requirements: not the user's rules, so never reported as unmet.
    gaps: Vec<Requirement>,
    /// The 0/1 "inside its minimum gap" column, once gaps are added.
    min_gap_column: Option<usize>,
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
        let tag_ids: Vec<Uuid> = constraints.tags.iter().map(|r| r.id).collect();
        let tagged: HashSet<(Uuid, Uuid)> = recipe_tag::repository::links_for_tags(pool, &tag_ids)
            .await?
            .into_iter()
            .map(|link| (link.recipe_id, link.tag_id))
            .collect();
        let mut grid = Grid::new(recipes, &uses, &tagged, constraints);
        grid.add_cook_gaps(recipes, &CookHistory::load(pool).await?);
        Ok(grid)
    }

    /// `uses` holds (recipe id, ingredient id) for every recipe using a must-use
    /// ingredient, and `tagged` (recipe id, tag id) for every recipe with a ruled tag.
    fn new(
        recipes: &[Recipe],
        uses: &HashSet<(Uuid, Uuid)>,
        tagged: &HashSet<(Uuid, Uuid)>,
        constraints: &RollConstraints,
    ) -> Grid {
        let ingredient_ids: Vec<Uuid> = constraints.ingredients.iter().map(|r| r.id).collect();
        let tag_ids: Vec<Uuid> = constraints.tags.iter().map(|r| r.id).collect();
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
                row.extend(
                    tag_ids
                        .iter()
                        .map(|&tag| i32::from(tagged.contains(&(r.id, tag)))),
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
        let first_tag = FIRST_INGREDIENT + constraints.ingredients.len();
        for (index, rule) in constraints.tags.iter().enumerate() {
            let requirement = counted(
                first_tag + index,
                1,
                1,
                rule.times,
                rule.bound,
                rule.importance,
            );
            rules.push((RuleFamily::Tag, index, requirement));
        }
        for (index, rule) in constraints.difficulties.iter().enumerate() {
            rules.push((RuleFamily::Difficulty, index, difficulty(rule)));
        }
        Grid {
            rows,
            rules,
            gaps: Vec::new(),
            min_gap_column: None,
        }
    }

    /// Appends an "inside its minimum gap" and an "overdue" column after every
    /// rule's column. Keeping recently cooked recipes out ranks above every
    /// preference the Roll page sends; pulling overdue ones in ranks below all
    /// of them.
    fn add_cook_gaps(&mut self, recipes: &[Recipe], history: &CookHistory) {
        let min_gap = self.rows.first().map_or(0, Vec::len);
        let overdue = min_gap + 1;
        for (row, recipe) in self.rows.iter_mut().zip(recipes) {
            row.push(i32::from(history.within_min_gap(recipe)));
            row.push(i32::from(history.overdue(recipe)));
        }
        let flagged = |column: usize| self.rows.iter().filter(|row| row[column] == 1).count();
        let (recent, due) = (flagged(min_gap), flagged(overdue));
        if recent > 0 {
            self.gaps.push(Requirement {
                attribute: min_gap,
                low: 1,
                high: 1,
                at_least: 0,
                at_most: Some(0),
                importance: solver::Importance::High,
            });
        }
        if due > 0 {
            self.gaps.push(Requirement {
                attribute: overdue,
                low: 1,
                high: 1,
                at_least: u32::try_from(due).unwrap_or(u32::MAX),
                at_most: None,
                importance: solver::Importance::Low,
            });
        }
        self.min_gap_column = Some(min_gap);
    }

    /// Whether the recipe at `index` was cooked inside its minimum gap.
    fn within_min_gap(&self, index: usize) -> bool {
        self.min_gap_column
            .is_some_and(|column| self.rows[index][column] == 1)
    }

    fn requirements(&self) -> Vec<Requirement> {
        self.rules
            .iter()
            .map(|(_, _, r)| *r)
            .chain(self.gaps.iter().copied())
            .collect()
    }

    /// How many of the `picks` (indexes into the recipes) match each rule.
    fn achieved(&self, picks: &[usize]) -> Vec<u32> {
        self.rules
            .iter()
            .map(|(_, _, r)| {
                let matched = picks.iter().filter(|&&i| r.matches(&self.rows[i])).count();
                u32::try_from(matched).unwrap_or(u32::MAX)
            })
            .collect()
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
                    Some(_) => (Bound::Exactly, r.at_least),
                    None => (Bound::AtLeast, r.at_least),
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
        Bound::Exactly => (times, Some(times)),
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

/// When each recipe was last cooked, and the day gaps are counted up to.
struct CookHistory {
    last_cooked: HashMap<Uuid, Date>,
    today: Date,
}

impl CookHistory {
    async fn load(pool: &PgPool) -> Result<CookHistory, AppError> {
        let last_cooked = cook_log::repository::last_cooks(pool)
            .await?
            .into_iter()
            .map(|cook| (cook.recipe_id, cook.cooked_on))
            .collect();
        Ok(CookHistory {
            last_cooked,
            today: OffsetDateTime::now_utc().date(),
        })
    }

    fn days_since(&self, recipe: &Recipe) -> Option<i64> {
        self.last_cooked
            .get(&recipe.id)
            .map(|&cooked| (self.today - cooked).whole_days())
    }

    /// A recipe never cooked is never inside its minimum gap.
    fn within_min_gap(&self, recipe: &Recipe) -> bool {
        match (recipe.min_gap_weeks, self.days_since(recipe)) {
            (Some(weeks), Some(days)) => days < i64::from(weeks) * 7,
            _ => false,
        }
    }

    /// A recipe never cooked is not overdue: the pull starts after its first
    /// logged cook, so setting a gap changes no roll until then.
    fn overdue(&self, recipe: &Recipe) -> bool {
        recipe.max_gap_weeks.is_some_and(|weeks| {
            self.days_since(recipe)
                .is_some_and(|days| days >= i64::from(weeks) * 7)
        })
    }
}

/// Seeds stay small and positive so they are easy to read off the plan and
/// type back in to repeat a roll.
fn fresh_seed() -> i64 {
    i64::from(rand::random::<u32>())
}

fn suggest_unpinning(error: AppError, pinned: &[PinnedDinner]) -> AppError {
    match error {
        AppError::Unprocessable(message) if !pinned.is_empty() => {
            AppError::Unprocessable(format!("{message} Or unpin a dinner."))
        }
        other => other,
    }
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
            min_gap_weeks: None,
            max_gap_weeks: None,
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
            tags: Vec::new(),
            day_limits: None,
        };
        let grid = Grid::new(&recipes, &uses, &HashSet::new(), &constraints);
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
        let grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
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
        let grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
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

    fn tagged_grid(
        recipes: &[Recipe],
        tag: Uuid,
        tagged: &[usize],
        constraints: &RollConstraints,
    ) -> Grid {
        let tagged: HashSet<(Uuid, Uuid)> = tagged.iter().map(|&i| (recipes[i].id, tag)).collect();
        Grid::new(recipes, &HashSet::new(), &tagged, constraints)
    }

    fn plain_recipes(count: usize) -> Vec<Recipe> {
        (0..count)
            .map(|_| recipe(Uuid::now_v7(), Uuid::now_v7(), 2))
            .collect()
    }

    #[test]
    fn a_required_tag_floor_counts_the_recipes_carrying_the_tag() {
        let (quick, recipes) = (Uuid::now_v7(), plain_recipes(8));
        let constraints = RollConstraints {
            tags: vec![rule(quick, 2, Bound::AtLeast, Importance::Required)],
            ..RollConstraints::default()
        };
        let grid = tagged_grid(&recipes, quick, &[1, 3, 4, 6], &constraints);
        for seed in 0..40 {
            let solution = solve(&grid, 4, seed).expect("four recipes carry the tag");
            assert!(solution.achieved[0] >= 2, "seed {seed}");
            assert!(grid.unmet(&solution.achieved).is_empty(), "seed {seed}");
        }
    }

    #[test]
    fn a_required_exactly_holds_both_bounds() {
        let (vegetarian, recipes) = (Uuid::now_v7(), plain_recipes(8));
        let constraints = RollConstraints {
            tags: vec![rule(vegetarian, 2, Bound::Exactly, Importance::Required)],
            ..RollConstraints::default()
        };
        let grid = tagged_grid(&recipes, vegetarian, &[0, 1, 2, 3, 4], &constraints);
        for seed in 0..40 {
            let solution = solve(&grid, 5, seed).expect("two of five tagged picks fit");
            assert_eq!(solution.achieved, [2], "seed {seed}");
        }
    }

    #[test]
    fn a_required_exactly_works_on_a_cuisine_too() {
        let (thai, italian, chicken) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
        let recipes: Vec<Recipe> = (0..6)
            .map(|n| recipe(if n < 4 { thai } else { italian }, chicken, 2))
            .collect();
        let constraints = RollConstraints {
            cuisines: vec![rule(thai, 3, Bound::Exactly, Importance::Required)],
            ..RollConstraints::default()
        };
        let grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
        for seed in 0..40 {
            let solution = solve(&grid, 5, seed).expect("three Thai and two Italian fit");
            assert_eq!(solution.achieved, [3], "seed {seed}");
        }
    }

    #[test]
    fn an_exactly_preference_that_cannot_hold_still_rolls_and_is_reported() {
        let (quick, recipes) = (Uuid::now_v7(), plain_recipes(6));
        let constraints = RollConstraints {
            tags: vec![rule(quick, 3, Bound::Exactly, Importance::Medium)],
            ..RollConstraints::default()
        };
        let grid = tagged_grid(&recipes, quick, &[0, 1], &constraints);
        let solution = solve(&grid, 4, 7).expect("preferences never fail a roll");
        assert_eq!(solution.achieved, [2]);
        assert_eq!(
            grid.unmet(&solution.achieved),
            [UnmetRule {
                family: RuleFamily::Tag,
                index: 0,
                bound: Bound::Exactly,
                times: 3,
                got: 2,
            }]
        );
    }

    #[test]
    fn an_exactly_preference_is_loosened_to_the_nearest_count_above_as_well_as_below() {
        let (quick, recipes) = (Uuid::now_v7(), plain_recipes(6));
        let constraints = RollConstraints {
            tags: vec![rule(quick, 2, Bound::Exactly, Importance::Medium)],
            ..RollConstraints::default()
        };
        let grid = tagged_grid(&recipes, quick, &[0, 1, 2, 3, 4], &constraints);
        for seed in 0..20 {
            let solution = solve(&grid, 4, seed).expect("preferences never fail a roll");
            assert_eq!(solution.achieved, [3], "seed {seed}");
        }
    }

    #[test]
    fn an_exactly_more_than_the_tagged_recipes_cannot_be_required() {
        let (quick, recipes) = (Uuid::now_v7(), plain_recipes(6));
        let constraints = RollConstraints {
            tags: vec![rule(quick, 3, Bound::Exactly, Importance::Required)],
            ..RollConstraints::default()
        };
        let grid = tagged_grid(&recipes, quick, &[0, 1], &constraints);
        assert_eq!(solve(&grid, 4, 0), Err(SolveError::Unsatisfiable));
    }

    #[test]
    fn a_required_exactly_zero_rules_the_recipes_out() {
        let (quick, recipes) = (Uuid::now_v7(), plain_recipes(5));
        let constraints = RollConstraints {
            tags: vec![rule(quick, 0, Bound::Exactly, Importance::Required)],
            ..RollConstraints::default()
        };
        let grid = tagged_grid(&recipes, quick, &[0, 1], &constraints);
        assert_eq!(fitting_rows(&grid.rows, &grid.requirements(), None, 3), 3);
    }

    #[test]
    fn the_residual_of_an_exactly_counts_the_kept_days_against_both_bounds() {
        let exactly_two = counted(
            FIRST_INGREDIENT,
            1,
            1,
            2,
            Bound::Exactly,
            Importance::Required,
        );
        let kept = [vec![0, 0, 3, 20, 1], vec![0, 0, 3, 20, 0]];
        let kept: Vec<&Vec<i32>> = kept.iter().collect();
        let left = residual(&[exactly_two], &kept);
        assert_eq!((left[0].at_least, left[0].at_most), (1, Some(1)));
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
        let grid = Grid::new(
            &recipes,
            &HashSet::new(),
            &HashSet::new(),
            &RollConstraints::default(),
        );
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
        let grid = Grid::new(
            &recipes,
            &HashSet::new(),
            &HashSet::new(),
            &RollConstraints::default(),
        );
        let plan = limit_plan();
        for seed in 0..20 {
            let solution = solver::roll(&Problem {
                recipes: grid.rows.clone(),
                requirements: grid.requirements(),
                slot_limits: open_slot_limits(&plan, &[5]),
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
        let grid = Grid::new(
            &recipes,
            &HashSet::new(),
            &HashSet::new(),
            &RollConstraints::default(),
        );
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
    fn achieved_counts_the_picks_matching_each_rule() {
        let thai = Uuid::now_v7();
        let beef = Uuid::now_v7();
        let recipes = vec![
            recipe(thai, beef, 1),
            recipe(thai, Uuid::now_v7(), 2),
            recipe(Uuid::now_v7(), beef, 5),
        ];
        let constraints = RollConstraints {
            cuisines: vec![rule(thai, 2, Bound::AtLeast, Importance::High)],
            proteins: vec![rule(beef, 1, Bound::AtMost, Importance::Medium)],
            ..RollConstraints::default()
        };
        let grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
        assert_eq!(grid.achieved(&[0, 1]), [2, 1]);
        assert_eq!(grid.achieved(&[0, 2]), [1, 2]);
        let unmet = grid.unmet(&grid.achieved(&[0, 2]));
        assert_eq!(unmet.len(), 2);
    }

    #[test]
    fn only_required_at_most_zero_rules_and_day_limits_rule_recipes_out() {
        let chicken = Uuid::now_v7();
        let recipes = vec![
            timed(1, 20),
            Recipe {
                main_protein_id: Some(chicken),
                ..timed(1, 20)
            },
            timed(5, 20),
            timed(1, 90),
        ];
        let constraints = RollConstraints {
            proteins: vec![
                rule(chicken, 0, Bound::AtMost, Importance::Required),
                rule(Uuid::now_v7(), 0, Bound::AtMost, Importance::High),
            ],
            ..RollConstraints::default()
        };
        let grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
        let requirements = grid.requirements();
        assert_eq!(fitting_rows(&grid.rows, &requirements, None, 5), 3);
        let plan = DayLimitPlan {
            default: DayLimits {
                max_difficulty: Some(DifficultyBand::Medium),
                max_time_mins: Some(30),
                ..DayLimits::default()
            },
            overrides: Vec::new(),
        };
        assert_eq!(fitting_rows(&grid.rows, &requirements, Some(&plan), 5), 1);
        let hard_day = DayLimitPlan {
            overrides: vec![DayLimitsAt {
                day: 2,
                limits: DayLimits::default(),
            }],
            ..plan
        };
        assert_eq!(
            fitting_rows(&grid.rows, &requirements, Some(&hard_day), 5),
            3
        );
        assert_eq!(
            fitting_rows(&grid.rows, &requirements, Some(&hard_day), 2),
            1
        );
    }

    #[test]
    fn pinned_dinners_count_toward_the_rules_and_are_not_picked_again() {
        let (thai, italian, chicken, beef) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let recipes = vec![
            recipe(thai, chicken, 2),
            recipe(thai, chicken, 3),
            recipe(thai, beef, 2),
            recipe(italian, chicken, 1),
            recipe(italian, beef, 4),
            recipe(italian, beef, 3),
            recipe(thai, beef, 5),
        ];
        let constraints = RollConstraints {
            cuisines: vec![rule(thai, 2, Bound::AtLeast, Importance::Required)],
            proteins: vec![rule(chicken, 1, Bound::AtMost, Importance::Required)],
            ..RollConstraints::default()
        };
        let grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
        let pinned = recipes[0].id;
        let kept: HashSet<Uuid> = [pinned].into_iter().collect();
        for seed in 0..30 {
            let (problem, candidates) =
                open_problem(&grid, &recipes, &kept, None, None, &[0, 2, 3], seed);
            let solution = solver::roll(&problem).expect("the rules fit around the pin");
            let open: Vec<&Recipe> = solution
                .picks
                .iter()
                .map(|&pick| &recipes[candidates[pick]])
                .collect();
            assert!(open.iter().all(|r| r.id != pinned), "seed {seed}");
            assert!(
                open.iter().all(|r| r.main_protein_id != Some(chicken)),
                "seed {seed}"
            );
            assert!(
                open.iter().any(|r| r.cuisine_id == Some(thai)),
                "seed {seed}"
            );
        }
    }

    #[test]
    fn with_nothing_kept_the_open_problem_is_the_plain_roll() {
        let recipes: Vec<Recipe> = (0..6).map(|n| timed(1 + n % 5, 20 + n)).collect();
        let grid = Grid::new(
            &recipes,
            &HashSet::new(),
            &HashSet::new(),
            &RollConstraints::default(),
        );
        let plan = limit_plan();
        let (problem, candidates) = open_problem(
            &grid,
            &recipes,
            &HashSet::new(),
            None,
            Some(&plan),
            &[0, 1, 2, 3],
            7,
        );
        assert_eq!(candidates, (0..6).collect::<Vec<_>>());
        assert_eq!(
            problem,
            Problem {
                recipes: grid.rows.clone(),
                requirements: grid.requirements(),
                slot_limits: (0..4).flat_map(|d| slot_limits_for(&plan, d)).collect(),
                num_to_pick: 4,
                seed: 7,
            }
        );
    }

    #[test]
    fn an_open_day_after_a_pinned_one_keeps_its_own_limits() {
        let limits = open_slot_limits(&limit_plan(), &[0, 2, 5]);
        assert!(limits.iter().all(|l| l.day < 3));
        assert!(limits
            .iter()
            .any(|l| l.day == 2 && l.attribute == DIFFICULTY && l.low == 3));
        assert!(limits
            .iter()
            .any(|l| l.day == 1 && l.attribute == DIFFICULTY && l.low == 1));
    }

    #[tokio::test]
    async fn a_week_with_every_day_pinned_needs_no_solver() {
        let recipes = vec![timed(1, 20), timed(2, 30)];
        let grid = Grid::new(
            &recipes,
            &HashSet::new(),
            &HashSet::new(),
            &RollConstraints::default(),
        );
        let kept: HashSet<Uuid> = recipes.iter().map(|r| r.id).collect();
        let picks = pick_open_days(&grid, &recipes, &kept, None, None, &[], 1)
            .await
            .expect("nothing to pick");
        assert!(picks.is_empty());
    }

    #[test]
    fn pins_must_fit_the_roll() {
        let recipes = vec![timed(1, 20), timed(2, 30)];
        let pin = |day: u16, recipe_id: Uuid| PinnedDinner { day, recipe_id };
        let ok = pinned_days(&recipes, &[pin(2, recipes[1].id)], 3).expect("valid pin");
        assert_eq!(ok.get(&2), Some(&1));
        let rejected = [
            vec![pin(3, recipes[0].id)],
            vec![pin(0, Uuid::now_v7())],
            vec![pin(0, recipes[0].id), pin(1, recipes[0].id)],
            vec![pin(0, recipes[0].id), pin(0, recipes[1].id)],
        ];
        for pins in rejected {
            assert!(
                matches!(
                    pinned_days(&recipes, &pins, 3),
                    Err(AppError::BadRequest(_))
                ),
                "{pins:?}"
            );
        }
    }

    async fn stored_recipe(pool: &PgPool, title: &str) -> Uuid {
        recipe::repository::create(
            pool,
            domain::recipe::InsertRecipe {
                household_id: None,
                title: title.to_string(),
                difficulty: 2,
                time_mins: 20,
                servings: 2,
                healthiness: None,
                main_protein_id: None,
                cuisine_id: None,
                min_gap_weeks: None,
                max_gap_weeks: None,
            },
        )
        .await
        .expect("recipe stored")
        .id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_roll_keeps_each_pinned_dinner_on_its_day(pool: PgPool) {
        let mut ids = Vec::new();
        for n in 0..8 {
            ids.push(stored_recipe(&pool, &format!("Dinner {n}")).await);
        }
        let request = |pinned: Vec<PinnedDinner>, seed: i64| RollRequest {
            household_id: None,
            start_date: None,
            num_days: 4,
            seed: Some(seed),
            constraints: RollConstraints::default(),
            pinned,
        };
        for seed in 0..10 {
            let pinned = vec![PinnedDinner {
                day: 2,
                recipe_id: ids[0],
            }];
            let preview = preview_roll(&pool, request(pinned, seed))
                .await
                .expect("rolls");
            let week: Vec<Uuid> = preview.meals.iter().map(|m| m.recipe_id).collect();
            assert_eq!(week[2], ids[0], "seed {seed}");
            assert_eq!(week.iter().collect::<HashSet<_>>().len(), 4, "seed {seed}");
        }
        let every_day: Vec<PinnedDinner> = (0..4)
            .map(|day| PinnedDinner {
                day,
                recipe_id: ids[usize::from(day) + 3],
            })
            .collect();
        let preview = preview_roll(&pool, request(every_day, 1))
            .await
            .expect("rolls");
        let week: Vec<Uuid> = preview.meals.iter().map(|m| m.recipe_id).collect();
        assert_eq!(week, ids[3..7]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn an_unsolvable_roll_suggests_unpinning_only_when_dinners_are_pinned(pool: PgPool) {
        let first = stored_recipe(&pool, "Dinner 0").await;
        stored_recipe(&pool, "Dinner 1").await;
        let request = |pinned: Vec<PinnedDinner>| RollRequest {
            household_id: None,
            start_date: None,
            num_days: 3,
            seed: Some(1),
            constraints: RollConstraints::default(),
            pinned,
        };
        let message = |result: Result<_, AppError>| match result {
            Err(AppError::Unprocessable(message)) => message,
            _ => panic!("expected an unprocessable roll"),
        };
        let pinned = vec![PinnedDinner {
            day: 0,
            recipe_id: first,
        }];
        assert!(message(preview_roll(&pool, request(pinned)).await).contains("unpin a dinner"));
        assert!(!message(preview_roll(&pool, request(Vec::new())).await).contains("unpin"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn pins_alone_over_a_required_maximum_fail_the_roll_but_not_a_preference(pool: PgPool) {
        let mut ids = Vec::new();
        for n in 0..4 {
            ids.push(stored_recipe(&pool, &format!("Dinner {n}")).await);
        }
        let request = |importance: Importance| RollRequest {
            household_id: None,
            start_date: None,
            num_days: 3,
            seed: Some(1),
            constraints: RollConstraints {
                difficulties: vec![DifficultyRule {
                    band: DifficultyBand::Easy,
                    times: 1,
                    bound: Bound::AtMost,
                    importance,
                }],
                ..RollConstraints::default()
            },
            pinned: (0..2)
                .map(|day| PinnedDinner {
                    day,
                    recipe_id: ids[usize::from(day)],
                })
                .collect(),
        };
        match preview_roll(&pool, request(Importance::Required)).await {
            Err(AppError::Unprocessable(message)) => assert!(message.contains("unpin a dinner")),
            _ => panic!("expected an unprocessable roll"),
        }
        assert!(preview_roll(&pool, request(Importance::Medium))
            .await
            .is_ok());
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

    fn history(cooks: &[(&Recipe, i64)]) -> CookHistory {
        let today = time::macros::date!(2026 - 11 - 15);
        CookHistory {
            last_cooked: cooks
                .iter()
                .map(|&(recipe, days_ago)| (recipe.id, today - Duration::days(days_ago)))
                .collect(),
            today,
        }
    }

    fn gapped(min: Option<i16>, max: Option<i16>) -> Recipe {
        Recipe {
            min_gap_weeks: min,
            max_gap_weeks: max,
            ..recipe(Uuid::now_v7(), Uuid::now_v7(), 2)
        }
    }

    fn gapped_grid(recipes: &[Recipe], history: &CookHistory) -> Grid {
        let mut grid = Grid::new(
            recipes,
            &HashSet::new(),
            &HashSet::new(),
            &RollConstraints::default(),
        );
        grid.add_cook_gaps(recipes, history);
        grid
    }

    #[test]
    fn the_minimum_gap_counts_whole_weeks_since_the_last_cook() {
        let gapped_one_week = gapped(Some(1), None);
        let unlimited = gapped(None, None);
        let cooks = history(&[(&gapped_one_week, 6), (&unlimited, 1)]);
        assert!(cooks.within_min_gap(&gapped_one_week));
        assert!(!cooks.within_min_gap(&unlimited));

        let a_week_ago = history(&[(&gapped_one_week, 7)]);
        assert!(!a_week_ago.within_min_gap(&gapped_one_week));
    }

    #[test]
    fn a_recipe_is_overdue_only_after_its_first_logged_cook() {
        let two_weeks = gapped(None, Some(2));
        assert!(!history(&[]).overdue(&two_weeks));
        assert!(!history(&[(&two_weeks, 13)]).overdue(&two_weeks));
        assert!(history(&[(&two_weeks, 14)]).overdue(&two_weeks));

        let unlimited = gapped(None, None);
        assert!(!history(&[(&unlimited, 30)]).overdue(&unlimited));
    }

    #[test]
    fn recipes_inside_their_minimum_gap_stay_out_while_others_can_fill_the_week() {
        let mut recipes: Vec<Recipe> = (0..8).map(|_| gapped(None, None)).collect();
        for recipe in &mut recipes[..3] {
            recipe.min_gap_weeks = Some(2);
        }
        let cooks: Vec<(&Recipe, i64)> = recipes[..3].iter().map(|r| (r, 3)).collect();
        let grid = gapped_grid(&recipes, &history(&cooks));
        for seed in 0..40 {
            let picks = solve(&grid, 4, seed).unwrap().picks;
            assert!(picks.iter().all(|&i| i >= 3), "seed {seed}: {picks:?}");
            assert!(grid.unmet(&grid.achieved(&picks)).is_empty());
        }
    }

    #[test]
    fn a_must_rule_brings_in_a_recipe_inside_its_gap_and_flags_it() {
        let thai = Uuid::now_v7();
        let mut recipes: Vec<Recipe> = (0..6).map(|_| gapped(None, None)).collect();
        recipes[0] = Recipe {
            cuisine_id: Some(thai),
            min_gap_weeks: Some(4),
            ..recipe(thai, Uuid::now_v7(), 2)
        };
        let constraints = RollConstraints {
            cuisines: vec![rule(thai, 1, Bound::AtLeast, Importance::Required)],
            ..RollConstraints::default()
        };
        let mut grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
        grid.add_cook_gaps(&recipes, &history(&[(&recipes[0], 2)]));

        let picks = solve(&grid, 3, 7).unwrap().picks;
        assert!(picks.contains(&0));
        assert!(grid.within_min_gap(0));
        assert!(grid.unmet(&grid.achieved(&picks)).is_empty());
    }

    #[test]
    fn overdue_recipes_are_pulled_into_the_week() {
        let mut recipes: Vec<Recipe> = (0..8).map(|_| gapped(None, None)).collect();
        for recipe in &mut recipes[..2] {
            recipe.max_gap_weeks = Some(2);
        }
        let cooks: Vec<(&Recipe, i64)> = recipes[..2].iter().map(|r| (r, 20)).collect();
        let grid = gapped_grid(&recipes, &history(&cooks));
        for seed in 0..40 {
            let picks = solve(&grid, 3, seed).unwrap().picks;
            assert!(
                picks.contains(&0) && picks.contains(&1),
                "seed {seed}: {picks:?}"
            );
        }
    }

    #[test]
    fn overdue_recipes_over_a_time_cap_still_roll() {
        let recipes: Vec<Recipe> = (0..8)
            .map(|n| Recipe {
                max_gap_weeks: (n < 3).then_some(2),
                time_mins: if n < 3 { 90 } else { 20 },
                ..gapped(None, None)
            })
            .collect();
        let cooks: Vec<(&Recipe, i64)> = recipes[..3].iter().map(|r| (r, 20)).collect();
        let grid = gapped_grid(&recipes, &history(&cooks));
        let plan = DayLimitPlan {
            default: DayLimits {
                max_time_mins: Some(30),
                ..DayLimits::default()
            },
            overrides: Vec::new(),
        };
        let picks = solver::roll(&Problem {
            recipes: grid.rows.clone(),
            requirements: grid.requirements(),
            slot_limits: (0..4).flat_map(|d| slot_limits_for(&plan, d)).collect(),
            num_to_pick: 4,
            seed: 1,
        })
        .unwrap()
        .picks;
        assert!(picks.iter().all(|&i| i >= 3));
    }

    #[test]
    fn the_gap_requirements_follow_the_rule_requirements() {
        let cuisine = Uuid::now_v7();
        let mut recipes = plain_recipes(4);
        recipes[0].min_gap_weeks = Some(2);
        let constraints = RollConstraints {
            cuisines: vec![rule(cuisine, 1, Bound::AtLeast, Importance::Medium)],
            ..RollConstraints::default()
        };
        let mut grid = Grid::new(&recipes, &HashSet::new(), &HashSet::new(), &constraints);
        let rule_requirement = grid.requirements()[0];
        grid.add_cook_gaps(&recipes, &history(&[(&recipes[0], 1)]));

        let requirements = grid.requirements();
        assert_eq!(requirements.len(), 2);
        assert_eq!(requirements[0], rule_requirement);
        assert_eq!(requirements[1].at_most, Some(0));
        assert_eq!(requirements[1].importance, solver::Importance::High);
    }

    async fn recipe_titled(pool: &PgPool, title: &str) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO recipe (title, difficulty, time_mins, servings) VALUES ($1, 1, 20, 2) RETURNING id",
            title,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn planned_meal_of(pool: &PgPool, recipe_id: Uuid) -> Uuid {
        let household_id = household::repository::default_household(pool)
            .await
            .unwrap()
            .id;
        let plan_id = sqlx::query_scalar!(
            "INSERT INTO meal_plan (household_id) VALUES ($1) RETURNING id",
            household_id,
        )
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "INSERT INTO planned_meal (meal_plan_id, recipe_id, day_index, scheduled_date)
             VALUES ($1, $2, 1, '2026-11-14')
             RETURNING id",
            plan_id,
            recipe_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_dinner_in_the_cooking_log_cannot_be_swapped_until_the_cook_is_undone(pool: PgPool) {
        let pasta = recipe_titled(&pool, "Pasta").await;
        let curry = recipe_titled(&pool, "Curry").await;
        let dinner = planned_meal_of(&pool, pasta).await;
        let cook = cook_log::repository::log(
            &pool,
            pasta,
            Some(dinner),
            time::macros::date!(2026 - 11 - 14),
        )
        .await
        .unwrap();

        let refused = swap(&pool, dinner, curry).await.unwrap_err();
        assert!(matches!(refused, AppError::Conflict(_)));
        let unchanged = planned_meal::repository::get_detail(&pool, dinner)
            .await
            .unwrap();
        assert_eq!(unchanged.recipe_id, pasta);

        cook_log::repository::delete(&pool, cook.id).await.unwrap();
        let swapped = swap(&pool, dinner, curry).await.unwrap();
        assert_eq!(swapped.recipe_id, curry);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_dinner_in_the_cooking_log_cannot_be_rerolled_until_the_cook_is_undone(pool: PgPool) {
        let pasta = recipe_titled(&pool, "Pasta").await;
        let curry = recipe_titled(&pool, "Curry").await;
        let dinner = planned_meal_of(&pool, pasta).await;
        let cook = cook_log::repository::log(
            &pool,
            pasta,
            Some(dinner),
            time::macros::date!(2026 - 11 - 14),
        )
        .await
        .unwrap();

        let refused = reroll(&pool, dinner).await.unwrap_err();
        assert!(matches!(refused, AppError::Conflict(_)));

        cook_log::repository::delete(&pool, cook.id).await.unwrap();
        let rerolled = reroll(&pool, dinner).await.unwrap();
        assert_eq!(rerolled.recipe_id, curry);
    }
}
