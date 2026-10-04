#ifndef MR_SOLVER_H
#define MR_SOLVER_H

#include <stdint.h>

typedef struct {
    int32_t cuisine;
    int32_t protein;
} Recipe;

typedef struct {
    int32_t cuisine;
    int32_t num_days;
} CuisineRequirement;

typedef struct {
    int32_t protein;
    int32_t num_days;
} ProteinRequirement;

typedef struct {
    Recipe* recipes;
    CuisineRequirement* cuisine_requirements;
    ProteinRequirement* protein_requirements;
    int32_t num_cuisine_requirements;
    int32_t num_protein_requirements;
    int32_t num_recipes;
    int32_t num_recipes_to_pick;
} Input;

typedef struct {
    int32_t* recipe_indices;
} Output;

int mr_solver_ping(Input* input, Output* output);

#endif
