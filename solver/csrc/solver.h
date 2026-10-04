#ifndef MR_SOLVER_H
#define MR_SOLVER_H

#include <stddef.h>
#include <stdint.h>

/*
 * The boundary between Rust and the C solver. Rust allocates every buffer on
 * both sides of the call; C only reads the inputs and writes recipe indices
 * into the output buffer it is handed. Only integers cross: lookup values
 * (cuisines, proteins) arrive as small positive codes assigned by the caller.
 * A code of 0 on a recipe means "unspecified"; 0 in a requirement means "any".
 *
 * The Rust mirror of these types lives in solver/src/lib.rs. Adding a
 * constraint family means: a struct here, a pointer + count pair in Input,
 * the same two additions in lib.rs, and a line in the api's roll service that
 * fills it. The layout test in lib.rs fails if the two sides disagree.
 */

/* Status codes returned by mr_solver_roll. */
enum {
    MR_OK = 0,
    /* Null pointers, negative counts, or more picks than recipes. */
    MR_ERR_INVALID_INPUT = 1,
    /* The requirements cannot all be met from the recipes given. */
    MR_ERR_UNSATISFIABLE = 2
};

typedef struct {
    int32_t cuisine;
    int32_t protein;
} Recipe;

/* "Cuisine X must appear on num_days of the week." */
typedef struct {
    int32_t cuisine;
    int32_t num_days;
} CuisineRequirement;

/* "Protein X must appear on num_days of the week." */
typedef struct {
    int32_t protein;
    int32_t num_days;
} ProteinRequirement;

typedef struct {
    const Recipe* recipes;
    int32_t num_recipes;
    const CuisineRequirement* cuisine_requirements;
    int32_t num_cuisine_requirements;
    const ProteinRequirement* protein_requirements;
    int32_t num_protein_requirements;
    int32_t num_recipes_to_pick;
    /* Seeds the solver's random choices so a roll can be reproduced. */
    uint64_t seed;
} Input;

typedef struct {
    /* num_recipes_to_pick slots; each receives an index into Input.recipes. */
    int32_t* recipe_indices;
} Output;

int32_t mr_solver_roll(const Input* input, Output* output);

/* sizeof(Input) as compiled on the C side, so Rust can check both layouts agree. */
size_t mr_solver_input_size(void);

#endif
