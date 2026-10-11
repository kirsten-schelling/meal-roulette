//! Safe Rust face of the C meal solver in `csrc/`.
//!
//! `Input`, `RawRequirement` and `Output` mirror `csrc/solver.h` field for field
//! (`#[repr(C)]` makes the memory layout identical). `roll` is the only entry
//! point and holds the only `unsafe` block in the workspace: it builds the
//! C-side `Input` from borrowed slices, hands over an output buffer it owns,
//! and checks what came back before anything else trusts it.

use std::fmt;

/// How much a requirement matters. `Required` must hold or the roll fails; the
/// others are met as far as the recipes allow, most important first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Importance {
    Low,
    Medium,
    High,
    #[default]
    Required,
}

impl Importance {
    fn code(self) -> i32 {
        match self {
            Importance::Low => 1,
            Importance::Medium => 2,
            Importance::High => 3,
            Importance::Required => 4,
        }
    }
}

/// "Between `at_least` and `at_most` of the picks have a value in
/// `low..=high` in column `attribute`."
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub attribute: usize,
    pub low: i32,
    pub high: i32,
    pub at_least: u32,
    pub at_most: Option<u32>,
    pub importance: Importance,
}

impl Requirement {
    pub fn matches(&self, row: &[i32]) -> bool {
        row.get(self.attribute)
            .is_some_and(|value| (self.low..=self.high).contains(value))
    }

    /// Whether `count` matching picks satisfies the requirement as asked.
    pub fn is_met(&self, count: u32) -> bool {
        count >= self.at_least && self.at_most.is_none_or(|max| count <= max)
    }
}

#[repr(C)]
struct RawRequirement {
    attribute: i32,
    low: i32,
    high: i32,
    min_count: i32,
    max_count: i32,
    importance: i32,
}

#[repr(C)]
struct Input {
    attributes: *const i32,
    num_recipes: i32,
    num_attributes: i32,
    requirements: *const RawRequirement,
    num_requirements: i32,
    num_recipes_to_pick: i32,
    seed: u64,
}

#[repr(C)]
struct Output {
    recipe_indices: *mut i32,
}

unsafe extern "C" {
    fn mr_solver_roll(input: *const Input, output: *mut Output) -> i32;
}

const MR_OK: i32 = 0;
const MR_ERR_INVALID_INPUT: i32 = 1;
const MR_ERR_UNSATISFIABLE: i32 = 2;
const MR_ERR_GAVE_UP: i32 = 4;
const MR_NO_LIMIT: i32 = -1;

/// Everything one roll needs: each recipe as a row of attribute values (all
/// rows the same length), the requirements over those columns, how many
/// recipes to pick, and the seed for the solver's random choices.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Problem {
    pub recipes: Vec<Vec<i32>>,
    pub requirements: Vec<Requirement>,
    pub num_to_pick: usize,
    pub seed: u64,
}

/// A roll's result: `picks` holds positions into `Problem::recipes`, and
/// `achieved[j]` how many picks match `Problem::requirements[j]`, so a caller
/// can tell which preferences had to be loosened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution {
    pub picks: Vec<usize>,
    pub achieved: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolveError {
    /// The solver rejected the input (also raised here for an empty pick,
    /// rows of different lengths, or a requirement on a missing column).
    InvalidInput,
    /// No selection from these recipes satisfies every required rule.
    Unsatisfiable,
    /// The required rules took too long to settle either way.
    GaveUp,
    /// A status code this wrapper does not know; carries the raw value.
    Unknown(i32),
    /// The solver wrote an index that is not a position in the recipe pool.
    BadIndex(i32),
    /// The solver picked the same recipe (by pool index) more than once.
    Duplicate(usize),
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolveError::InvalidInput => f.write_str("the solver rejected its input"),
            SolveError::Unsatisfiable => {
                f.write_str("no selection of recipes satisfies every required rule")
            }
            SolveError::GaveUp => f.write_str("the solver ran out of steps before settling"),
            SolveError::Unknown(code) => write!(f, "the solver failed with status {code}"),
            SolveError::BadIndex(index) => {
                write!(
                    f,
                    "the solver returned recipe index {index}, which is out of range"
                )
            }
            SolveError::Duplicate(index) => {
                write!(f, "the solver picked recipe index {index} more than once")
            }
        }
    }
}

impl std::error::Error for SolveError {}

fn raw(requirement: &Requirement) -> Result<RawRequirement, SolveError> {
    let count = |n: u32| i32::try_from(n).map_err(|_| SolveError::InvalidInput);
    Ok(RawRequirement {
        attribute: i32::try_from(requirement.attribute).map_err(|_| SolveError::InvalidInput)?,
        low: requirement.low,
        high: requirement.high,
        min_count: count(requirement.at_least)?,
        max_count: match requirement.at_most {
            Some(max) => count(max)?,
            None => MR_NO_LIMIT,
        },
        importance: requirement.importance.code(),
    })
}

/// Run the solver. On success the picks are `problem.num_to_pick` distinct
/// positions into `problem.recipes`, in day order.
pub fn roll(problem: &Problem) -> Result<Solution, SolveError> {
    let pool_size = problem.recipes.len();
    if problem.num_to_pick == 0 {
        return Err(SolveError::InvalidInput);
    }
    if problem.num_to_pick > pool_size {
        return Err(SolveError::Unsatisfiable);
    }
    let num_attributes = problem.recipes[0].len();
    if problem
        .recipes
        .iter()
        .any(|row| row.len() != num_attributes)
        || problem
            .requirements
            .iter()
            .any(|r| r.attribute >= num_attributes)
    {
        return Err(SolveError::InvalidInput);
    }
    let count = |len: usize| i32::try_from(len).map_err(|_| SolveError::InvalidInput);

    let attributes: Vec<i32> = problem.recipes.concat();
    let requirements = problem
        .requirements
        .iter()
        .map(raw)
        .collect::<Result<Vec<_>, _>>()?;
    let input = Input {
        attributes: attributes.as_ptr(),
        num_recipes: count(pool_size)?,
        num_attributes: count(num_attributes)?,
        requirements: requirements.as_ptr(),
        num_requirements: count(requirements.len())?,
        num_recipes_to_pick: count(problem.num_to_pick)?,
        seed: problem.seed,
    };
    let mut picks = vec![0i32; problem.num_to_pick];
    let mut output = Output {
        recipe_indices: picks.as_mut_ptr(),
    };

    // SAFETY: every pointer in `input` borrows a Vec that lives until the end
    // of this function, and each is paired with its exact length (the
    // attribute grid holds num_recipes * num_attributes values). The output
    // buffer has `num_recipes_to_pick` slots, which is the contract in
    // solver.h; C writes only into it and keeps no pointer afterwards.
    let status = unsafe { mr_solver_roll(&input, &mut output) };

    match status {
        MR_OK => {}
        MR_ERR_INVALID_INPUT => return Err(SolveError::InvalidInput),
        MR_ERR_UNSATISFIABLE => return Err(SolveError::Unsatisfiable),
        MR_ERR_GAVE_UP => return Err(SolveError::GaveUp),
        other => return Err(SolveError::Unknown(other)),
    }

    let mut seen = vec![false; pool_size];
    let picks = picks
        .iter()
        .map(|&raw| {
            let index = usize::try_from(raw)
                .ok()
                .filter(|&index| index < pool_size)
                .ok_or(SolveError::BadIndex(raw))?;
            if std::mem::replace(&mut seen[index], true) {
                return Err(SolveError::Duplicate(index));
            }
            Ok(index)
        })
        .collect::<Result<Vec<usize>, _>>()?;
    let achieved = problem
        .requirements
        .iter()
        .map(|r| {
            let matching = picks
                .iter()
                .filter(|&&i| r.matches(&problem.recipes[i]))
                .count();
            u32::try_from(matching).unwrap_or(u32::MAX)
        })
        .collect();
    Ok(Solution { picks, achieved })
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" {
        fn mr_solver_input_size() -> usize;
        fn mr_solver_requirement_size() -> usize;
    }

    const CUISINE: usize = 0;
    const PROTEIN: usize = 1;
    const DIFFICULTY: usize = 2;

    fn recipe(cuisine: i32, protein: i32, difficulty: i32) -> Vec<i32> {
        vec![cuisine, protein, difficulty]
    }

    fn pool() -> Vec<Vec<i32>> {
        vec![recipe(1, 1, 1), recipe(2, 2, 3), recipe(3, 3, 5)]
    }

    fn rule(attribute: usize, value: i32, at_least: u32, importance: Importance) -> Requirement {
        Requirement {
            attribute,
            low: value,
            high: value,
            at_least,
            at_most: None,
            importance,
        }
    }

    fn at_most(attribute: usize, value: i32, max: u32, importance: Importance) -> Requirement {
        Requirement {
            at_least: 0,
            at_most: Some(max),
            ..rule(attribute, value, 0, importance)
        }
    }

    /// Every cuisine/protein pair from 1 to 3, plus a few repeats, at mixed difficulties.
    fn mixed_pool() -> Vec<Vec<i32>> {
        let mut recipes: Vec<Vec<i32>> = (1..=3)
            .flat_map(|cuisine| (1..=3).map(move |protein| recipe(cuisine, protein, protein + 1)))
            .collect();
        recipes.extend([recipe(1, 1, 5), recipe(2, 3, 1), recipe(3, 2, 2)]);
        recipes
    }

    fn mixed_problem(seed: u64) -> Problem {
        Problem {
            recipes: mixed_pool(),
            requirements: vec![
                rule(CUISINE, 1, 2, Importance::Required),
                rule(CUISINE, 2, 1, Importance::Required),
                rule(PROTEIN, 3, 2, Importance::Required),
                rule(PROTEIN, 1, 1, Importance::Required),
            ],
            num_to_pick: 5,
            seed,
        }
    }

    fn matching(problem: &Problem, solution: &Solution, r: &Requirement) -> usize {
        solution
            .picks
            .iter()
            .filter(|&&i| r.matches(&problem.recipes[i]))
            .count()
    }

    #[test]
    fn layouts_match_the_c_side() {
        assert_eq!(
            unsafe { mr_solver_input_size() },
            std::mem::size_of::<Input>()
        );
        assert_eq!(
            unsafe { mr_solver_requirement_size() },
            std::mem::size_of::<RawRequirement>()
        );
    }

    #[test]
    fn picks_one_recipe_from_the_pool_without_requirements() {
        let problem = Problem {
            recipes: pool(),
            num_to_pick: 1,
            seed: 42,
            ..Problem::default()
        };
        let solution = roll(&problem).expect("one unconstrained pick always succeeds");
        assert_eq!(solution.picks.len(), 1);
        assert!(solution.picks[0] < 3);
    }

    #[test]
    fn rejects_an_empty_pick_before_calling_c() {
        let problem = Problem {
            recipes: pool(),
            ..Problem::default()
        };
        assert_eq!(roll(&problem), Err(SolveError::InvalidInput));
    }

    #[test]
    fn more_picks_than_recipes_is_unsatisfiable() {
        let problem = Problem {
            recipes: pool(),
            num_to_pick: 4,
            ..Problem::default()
        };
        assert_eq!(roll(&problem), Err(SolveError::Unsatisfiable));
    }

    #[test]
    fn a_requirement_on_a_missing_column_is_rejected() {
        let problem = Problem {
            recipes: pool(),
            requirements: vec![rule(7, 1, 1, Importance::Required)],
            num_to_pick: 1,
            seed: 0,
        };
        assert_eq!(roll(&problem), Err(SolveError::InvalidInput));
    }

    #[test]
    fn never_picks_the_same_recipe_twice() {
        let recipes = mixed_pool();
        for seed in 0..100 {
            let problem = Problem {
                recipes: recipes.clone(),
                num_to_pick: recipes.len(),
                seed,
                ..Problem::default()
            };
            let mut picks = roll(&problem)
                .expect("picking the whole pool always succeeds")
                .picks;
            picks.sort_unstable();
            assert_eq!(picks, (0..recipes.len()).collect::<Vec<_>>(), "seed {seed}");
        }
    }

    #[test]
    fn meets_every_required_rule() {
        for seed in 0..100 {
            let problem = mixed_problem(seed);
            let solution = roll(&problem).expect("the mixed problem has a solution");
            for (r, &got) in problem.requirements.iter().zip(&solution.achieved) {
                assert!(r.is_met(got), "seed {seed}: {r:?} matched {got}");
                assert_eq!(matching(&problem, &solution, r), got as usize);
            }
        }
    }

    #[test]
    fn pairs_rules_across_different_recipes() {
        let (thai, italian, chicken, beef) = (1, 2, 1, 2);
        for seed in 0..50 {
            let problem = Problem {
                recipes: vec![recipe(thai, chicken, 2), recipe(italian, beef, 2)],
                requirements: vec![
                    rule(CUISINE, thai, 1, Importance::Required),
                    rule(PROTEIN, beef, 1, Importance::Required),
                ],
                num_to_pick: 2,
                seed,
            };
            let mut picks = roll(&problem)
                .expect("Thai chicken and Italian beef meet both rules")
                .picks;
            picks.sort_unstable();
            assert_eq!(picks, [0, 1], "seed {seed}");
        }
    }

    #[test]
    fn the_same_seed_rolls_the_same_week() {
        let first = roll(&mixed_problem(7)).expect("the mixed problem has a solution");
        assert_eq!(roll(&mixed_problem(7)), Ok(first));
    }

    #[test]
    fn too_few_recipes_of_a_protein_is_unsatisfiable() {
        let problem = Problem {
            recipes: pool(),
            requirements: vec![rule(PROTEIN, 1, 2, Importance::Required)],
            num_to_pick: 2,
            seed: 0,
        };
        assert_eq!(roll(&problem), Err(SolveError::Unsatisfiable));
    }

    #[test]
    fn a_count_beyond_the_week_is_unsatisfiable() {
        let problem = Problem {
            recipes: pool(),
            requirements: vec![rule(CUISINE, 1, u32::MAX >> 1, Importance::Required)],
            num_to_pick: 2,
            seed: 0,
        };
        assert_eq!(roll(&problem), Err(SolveError::Unsatisfiable));
    }

    #[test]
    fn at_most_caps_how_often_a_value_appears() {
        for seed in 0..50 {
            let problem = Problem {
                recipes: mixed_pool(),
                requirements: vec![at_most(PROTEIN, 1, 1, Importance::Required)],
                num_to_pick: 6,
                seed,
            };
            let solution = roll(&problem).expect("there are enough non-chicken recipes");
            assert!(solution.achieved[0] <= 1, "seed {seed}");
        }
    }

    #[test]
    fn a_difficulty_band_is_a_range_of_values() {
        let hard = Requirement {
            attribute: DIFFICULTY,
            low: 4,
            high: 5,
            at_least: 2,
            at_most: None,
            importance: Importance::Required,
        };
        for seed in 0..50 {
            let problem = Problem {
                recipes: mixed_pool(),
                requirements: vec![hard],
                num_to_pick: 4,
                seed,
            };
            let solution = roll(&problem).expect("the pool has 4 hard recipes");
            assert!(solution.achieved[0] >= 2, "seed {seed}");
        }
    }

    #[test]
    fn a_preference_is_kept_when_it_fits() {
        for seed in 0..50 {
            let problem = Problem {
                recipes: mixed_pool(),
                requirements: vec![rule(CUISINE, 3, 3, Importance::Medium)],
                num_to_pick: 4,
                seed,
            };
            let solution = roll(&problem).expect("preferences never fail a roll");
            assert!(solution.achieved[0] >= 3, "seed {seed}");
        }
    }

    #[test]
    fn a_preference_that_cannot_fit_is_loosened_not_failed() {
        let problem = Problem {
            recipes: vec![
                recipe(1, 1, 1),
                recipe(1, 2, 2),
                recipe(2, 1, 3),
                recipe(3, 3, 4),
            ],
            requirements: vec![rule(CUISINE, 1, 3, Importance::High)],
            num_to_pick: 3,
            seed: 5,
        };
        let solution = roll(&problem).expect("preferences never fail a roll");
        assert_eq!(solution.achieved[0], 2);
        assert!(!problem.requirements[0].is_met(2));
    }

    #[test]
    fn a_required_rule_beats_a_preference() {
        for seed in 0..30 {
            let problem = Problem {
                recipes: mixed_pool(),
                requirements: vec![
                    rule(PROTEIN, 1, 2, Importance::High),
                    at_most(PROTEIN, 1, 0, Importance::Required),
                ],
                num_to_pick: 3,
                seed,
            };
            let solution = roll(&problem).expect("the required rule alone fits");
            assert_eq!(solution.achieved, [0, 0], "seed {seed}");
        }
    }

    #[test]
    fn the_more_important_preference_wins_a_conflict() {
        for seed in 0..30 {
            let problem = Problem {
                recipes: mixed_pool(),
                requirements: vec![
                    at_most(CUISINE, 2, 0, Importance::Low),
                    rule(CUISINE, 2, 2, Importance::High),
                ],
                num_to_pick: 4,
                seed,
            };
            let solution = roll(&problem).expect("preferences never fail a roll");
            assert_eq!(solution.achieved[1], 2, "seed {seed}");
        }
    }
}
