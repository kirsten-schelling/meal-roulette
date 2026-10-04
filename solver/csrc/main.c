// Host test harness: a standalone stand-in for the Rust caller so you can
// compile and run the solver without the API and watch its output. This file
// is NOT part of the FFI build (issue #6) — build.rs will compile solver.c
// alone; `main` would clash there. `just solver` compiles both together.
//
// Logs go to stderr, the solver's result goes to stdout, so you can tell the
// two apart when redirecting.
//
// Input format (plain text, so no JSON parser is needed):
//   line 1  : how many recipes to pick
//   then    : one "cuisine protein" pair per recipe, whitespace-separated
// num_recipes is derived from how many pairs are read, the same way Rust will
// set it from the number of rows it fetched.

#include <stdio.h>
#include <stdlib.h>

#include "solver.h"

int main(int argc, char **argv) {
    const char *input_path = argc > 1 ? argv[1] : "input.txt";

    FILE *f = fopen(input_path, "r");
    if (!f) {
        fprintf(stderr, "[harness] could not open input: %s\n", input_path);
        return 1;
    }

    int want = 0;
    if (fscanf_s(f, "%d", &want) != 1) {
        fprintf(stderr, "[harness] input is missing the pick count on line 1\n");
        fclose(f);
        return 1;
    }

    CuisineRequirement *cuisine_requirements = NULL;
    int cuisine_count = 0;
    int cuisine_cap = 0;
    int cuisine, num_days;
    while (fscanf_s(f, "%d %d", &cuisine, &num_days) == 2) {
        if (cuisine_count == cuisine_cap) {
            cuisine_cap = cuisine_cap ? cuisine_cap * 2 : 8;
            cuisine_requirements = realloc(cuisine_requirements, sizeof(CuisineRequirement) * (size_t)cuisine_cap);
            if (!cuisine_requirements) {
                fprintf(stderr, "[harness] out of memory reading cuisine requirements\n");
                fclose(f);
                return 1;
            }
        }
        cuisine_requirements[cuisine_count].cuisine = cuisine;
        cuisine_requirements[cuisine_count].num_days = num_days;
        cuisine_count++;
    }

    while (fscanf_s(f, "-", &cuisine) == 1) {}

    ProteinRequirement *protein_requirements = NULL;
    int protein_count = 0;
    int protein_cap = 0;
    int protein2, num_days2;
    while (fscanf_s(f, "%d %d", &protein2, &num_days2) == 2) {
        if (protein_count == protein_cap) {
            protein_cap = protein_cap ? protein_cap * 2 : 8;
            protein_requirements = realloc(protein_requirements, sizeof(ProteinRequirement) * (size_t)protein_cap);
        }
        protein_requirements[protein_count].protein = protein2;
        protein_requirements[protein_count].num_days = num_days2;
        protein_count++;
    }

    while (fscanf_s(f, "-", &protein2) == 1) {}

    Recipe *recipes = NULL;
    int count = 0;
    int cap = 0;
    int cuisine2, protein;
    while (fscanf_s(f, "%d %d", &cuisine2, &protein) == 2) {
        if (count == cap) {
            cap = cap ? cap * 2 : 8;
            recipes = realloc(recipes, sizeof(Recipe) * (size_t)cap);
            if (!recipes) {
                fprintf(stderr, "[harness] out of memory reading recipes\n");
                fclose(f);
                return 1;
            }
        }
        recipes[count].cuisine = cuisine2;
        recipes[count].protein = protein;
        count++;
    }
    fclose(f);

    Input input = {
        .recipes = recipes,
        .cuisine_requirements = cuisine_requirements,
        .protein_requirements = protein_requirements,
        .num_cuisine_requirements = cuisine_count,
        .num_protein_requirements = protein_count,
        .num_recipes = count,
        .num_recipes_to_pick = want
    };
    fprintf(stderr, "[harness] read %d recipes from %s\n", count, input_path);

    Output output = {
        .recipe_indices = malloc(sizeof(int32_t) * (size_t)input.num_recipes_to_pick)
    };

    int status = mr_solver_ping(&input, &output);
    printf("mr_solver_ping() -> %d\n", status);

    free(output.recipe_indices);
    free(recipes);
    return status;
}
