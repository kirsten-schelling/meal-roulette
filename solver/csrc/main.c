// Host test harness: a standalone stand-in for the Rust caller so you can
// compile and run the solver without the API and watch its output. This file
// is NOT part of the FFI build (issue #6) — build.rs will compile solver.c
// alone; `main` would clash there. `just solver` compiles both together.
//
// Logs go to stderr, the solver's result goes to stdout, so you can tell the
// two apart when redirecting.
//
// Input format (plain text, so no JSON parser is needed):
//   line 1 : how many recipes to pick
//   line 2 : how many attribute columns each recipe has
//   then   : one requirement per line, "attribute low high min max importance"
//            (max -1 = no limit; importance 1 low .. 4 required), ended by "-"
//   then   : one recipe per line, its attribute values in column order
// num_recipes is derived from how many rows are read, the same way Rust sets
// it from the number of rows it fetched.

#include <stdio.h>
#include <stdlib.h>

#include "solver.h"

static void *grow(void *items, int *cap, size_t size) {
    *cap = *cap ? *cap * 2 : 8;
    return realloc(items, size * (size_t)*cap);
}

int main(int argc, char **argv) {
    const char *input_path = argc > 1 ? argv[1] : "input.txt";

    FILE *f = fopen(input_path, "r");
    if (!f) {
        fprintf(stderr, "[harness] could not open input: %s\n", input_path);
        return 1;
    }

    int want = 0;
    int num_attributes = 0;
    if (fscanf_s(f, "%d %d", &want, &num_attributes) != 2 || num_attributes < 1) {
        fprintf(stderr, "[harness] input needs the pick count and the column count first\n");
        fclose(f);
        return 1;
    }

    Requirement *requirements = NULL;
    int requirement_count = 0;
    int requirement_cap = 0;
    Requirement r;
    while (fscanf_s(f, "%d %d %d %d %d %d", &r.attribute, &r.low, &r.high, &r.min_count,
               &r.max_count, &r.importance) == 6) {
        if (requirement_count == requirement_cap) {
            requirements = grow(requirements, &requirement_cap, sizeof(Requirement));
            if (!requirements) {
                fprintf(stderr, "[harness] out of memory reading requirements\n");
                fclose(f);
                return 1;
            }
        }
        requirements[requirement_count++] = r;
    }
    char separator[2] = {0};
    fscanf_s(f, " %1[-]", separator, (unsigned)sizeof(separator));

    int32_t *attributes = NULL;
    int value_count = 0;
    int value_cap = 0;
    int value;
    while (fscanf_s(f, "%d", &value) == 1) {
        if (value_count == value_cap) {
            attributes = grow(attributes, &value_cap, sizeof(int32_t));
            if (!attributes) {
                fprintf(stderr, "[harness] out of memory reading recipes\n");
                fclose(f);
                return 1;
            }
        }
        attributes[value_count++] = value;
    }
    fclose(f);
    int count = value_count / num_attributes;

    Input input = {
        .attributes = attributes,
        .num_recipes = count,
        .num_attributes = num_attributes,
        .requirements = requirements,
        .num_requirements = requirement_count,
        .num_recipes_to_pick = want,
        .seed = 1
    };
    fprintf(stderr, "[harness] read %d recipes and %d requirements from %s\n", count,
        requirement_count, input_path);

    Output output = {
        .recipe_indices = malloc(sizeof(int32_t) * (size_t)(want > 0 ? want : 1))
    };

    int status = mr_solver_roll(&input, &output);
    printf("mr_solver_roll() -> %d\n", status);
    if (status == MR_OK) {
        printf("Selected recipes:\n");
        for (int i = 0; i < want; i++) {
            int32_t index = output.recipe_indices[i];
            printf("Index: %d  Values:", index);
            for (int a = 0; a < num_attributes; a++) {
                printf(" %d", attributes[(size_t)index * (size_t)num_attributes + a]);
            }
            printf("\n");
        }
    }

    free(output.recipe_indices);
    free(attributes);
    free(requirements);
    return status;
}
