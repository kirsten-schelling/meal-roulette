//! Safe Rust face of the C meal solver in `csrc/`.
//!
//! The types below mirror `csrc/solver.h` field for field (`#[repr(C)]` makes
//! the memory layout identical). `roll` is the only entry point and holds the
//! only `unsafe` block in the workspace: it builds the C-side `Input` from
//! borrowed slices, hands over an output buffer it owns, and checks what came
//! back before anything else trusts it.

use std::fmt;

/// One recipe as the solver sees it: lookup values as caller-assigned codes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Recipe {
    pub cuisine: i32,
    pub protein: i32,
}

/// "Cuisine `cuisine` must appear on `num_days` days."
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CuisineRequirement {
    pub cuisine: i32,
    pub num_days: i32,
}

/// "Protein `protein` must appear on `num_days` days."
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProteinRequirement {
    pub protein: i32,
    pub num_days: i32,
}

#[repr(C)]
struct Input {
    recipes: *const Recipe,
    num_recipes: i32,
    cuisine_requirements: *const CuisineRequirement,
    num_cuisine_requirements: i32,
    protein_requirements: *const ProteinRequirement,
    num_protein_requirements: i32,
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

/// Everything one roll needs: the recipe pool, every constraint family, how
/// many recipes to pick, and the seed for the solver's random choices.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Problem {
    pub recipes: Vec<Recipe>,
    pub cuisine_requirements: Vec<CuisineRequirement>,
    pub protein_requirements: Vec<ProteinRequirement>,
    pub num_to_pick: usize,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolveError {
    /// The solver rejected the input (also raised here for an empty pick or
    /// more picks than recipes, before C is called).
    InvalidInput,
    /// No selection from these recipes satisfies every requirement.
    Unsatisfiable,
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
                f.write_str("no selection of recipes satisfies every requirement")
            }
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

/// Run the solver. On success the result holds `problem.num_to_pick` distinct
/// positions into `problem.recipes`, in slot order.
pub fn roll(problem: &Problem) -> Result<Vec<usize>, SolveError> {
    let pool_size = problem.recipes.len();
    if problem.num_to_pick == 0 {
        return Err(SolveError::InvalidInput);
    }
    if problem.num_to_pick > pool_size {
        return Err(SolveError::Unsatisfiable);
    }
    let count = |len: usize| i32::try_from(len).map_err(|_| SolveError::InvalidInput);

    let input = Input {
        recipes: problem.recipes.as_ptr(),
        num_recipes: count(pool_size)?,
        cuisine_requirements: problem.cuisine_requirements.as_ptr(),
        num_cuisine_requirements: count(problem.cuisine_requirements.len())?,
        protein_requirements: problem.protein_requirements.as_ptr(),
        num_protein_requirements: count(problem.protein_requirements.len())?,
        num_recipes_to_pick: count(problem.num_to_pick)?,
        seed: problem.seed,
    };
    let mut picks = vec![0i32; problem.num_to_pick];
    let mut output = Output {
        recipe_indices: picks.as_mut_ptr(),
    };

    // SAFETY: every pointer in `input` borrows a Vec owned by `problem`, which
    // outlives the call, and each is paired with its exact length. The output
    // buffer has `num_recipes_to_pick` slots, which is the contract in
    // solver.h; C writes only into it and keeps no pointer afterwards.
    let status = unsafe { mr_solver_roll(&input, &mut output) };

    match status {
        MR_OK => {}
        MR_ERR_INVALID_INPUT => return Err(SolveError::InvalidInput),
        MR_ERR_UNSATISFIABLE => return Err(SolveError::Unsatisfiable),
        other => return Err(SolveError::Unknown(other)),
    }

    let mut seen = vec![false; pool_size];
    picks
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
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" {
        fn mr_solver_input_size() -> usize;
    }

    fn pool() -> Vec<Recipe> {
        vec![
            Recipe {
                cuisine: 1,
                protein: 1,
            },
            Recipe {
                cuisine: 2,
                protein: 2,
            },
            Recipe {
                cuisine: 3,
                protein: 3,
            },
        ]
    }

    #[test]
    fn input_layout_matches_the_c_side() {
        let c_size = unsafe { mr_solver_input_size() };
        assert_eq!(c_size, std::mem::size_of::<Input>());
    }

    #[test]
    fn picks_one_recipe_from_the_pool_without_requirements() {
        let problem = Problem {
            recipes: pool(),
            num_to_pick: 1,
            seed: 42,
            ..Problem::default()
        };
        let picks = roll(&problem).expect("one unconstrained pick always succeeds");
        assert_eq!(picks.len(), 1);
        assert!(picks[0] < 3);
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
}
