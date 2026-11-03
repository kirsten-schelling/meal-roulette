#ifndef MR_SOLVER_H
#define MR_SOLVER_H

#include <stddef.h>
#include <stdint.h>

/*
 * The boundary between Rust and the C solver. Rust allocates every buffer on
 * both sides of the call; C only reads the inputs and writes recipe indices
 * into the output buffer it is handed. Only integers cross.
 *
 * Recipes arrive as a grid of attribute values, one row per recipe and one
 * column per attribute (cuisine code, protein code, difficulty, "uses
 * ingredient X" as 0/1, ...). Every rule is the same Requirement shape over one
 * column. Rust decides what the columns mean, so a new kind of rule is a new
 * column and/or requirement on the Rust side; nothing here changes.
 *
 * The Rust mirror of these types lives in solver/src/lib.rs. The layout tests
 * there fail if the two sides disagree.
 */

/* Status codes returned by mr_solver_roll. */
enum {
    MR_OK = 0,
    /* Null pointers, negative counts, an attribute out of range, a rule whose
       min_count exceeds its max_count, no picks, or more picks than recipes. */
    MR_ERR_INVALID_INPUT = 1,
    /* The required rules cannot all be met from the recipes given. */
    MR_ERR_UNSATISFIABLE = 2,
    /* The solver could not allocate its working memory. */
    MR_ERR_NO_MEMORY = 3,
    /* The required rules took too long to settle either way. */
    MR_ERR_GAVE_UP = 4
};

/* How much a requirement matters. Required ones must hold; the others are kept
   most important first, as far as the recipes allow. */
enum {
    MR_LOW = 1,
    MR_MEDIUM = 2,
    MR_HIGH = 3,
    MR_REQUIRED = 4
};

/* max_count value for "no upper limit". */
#define MR_NO_LIMIT (-1)

/* "Between min_count and max_count of the picks have a value in [low, high]
   in column `attribute`." */
typedef struct {
    int32_t attribute;
    int32_t low;
    int32_t high;
    int32_t min_count;
    int32_t max_count;
    int32_t importance;
} Requirement;

typedef struct {
    /* num_recipes rows of num_attributes values: recipe r's value for column a
       is attributes[r * num_attributes + a]. */
    const int32_t* attributes;
    int32_t num_recipes;
    int32_t num_attributes;
    const Requirement* requirements;
    int32_t num_requirements;
    int32_t num_recipes_to_pick;
    /* Seeds the solver's random choices so a roll can be reproduced. */
    uint64_t seed;
} Input;

typedef struct {
    /* num_recipes_to_pick slots; each receives a row index into the attribute
       grid. No row is picked twice. */
    int32_t* recipe_indices;
} Output;

int32_t mr_solver_roll(const Input* input, Output* output);

/* sizeof(Input) and sizeof(Requirement) as compiled on the C side, so Rust can
   check both layouts agree. */
size_t mr_solver_input_size(void);
size_t mr_solver_requirement_size(void);

#endif
