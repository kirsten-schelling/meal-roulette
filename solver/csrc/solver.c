// Meal Roulette roller / constraint solver (portable C). Write the roll logic
// in this file: pick recipes for the week under the given constraints.
//
// Design and FFI plumbing (the cc/build.rs/extern-"C" scaffold) live in issue
// #6:
//   https://github.com/kirsten-schelling/meal-roulette/issues/6
//
// Boundary rules that keep this embedded-friendly and host-testable:
//   1. Rust (the caller) allocates every buffer, both the inputs and the output
//      you write into. C never mallocs anything that crosses the boundary.
//   2. Only ints / indices cross the boundary. No strings, no DB, no pointers
//      back into Rust. C picks recipe indices into the array it is given.
//   3. The RNG seed is passed in, so a roll is deterministic and reproducible.
//   4. Return a status code; write the picks through an out-parameter.
//
// Start by proving the plumbing end to end with this trivial function, then
// replace it with the real solver.

// Plan:
// 1. We take the requirements (eg cuisine x times per week), and then select
// random indexes between 1 and the number of days
// 2. Everything should be +1 of what index is was put in as, since 0 will be a
// placeholder for "random" in the shuffler
// 3. Once all the requirements are filled in to their random indexes, we can
// then select a random recipe base d on the requirements form each day
// Basically we filter down the list to match the requirements of the meal, and
// select randomly from what's left We repeat that for every remaining day To
// check, we should run a loop of validation. Where we take the requirements,
// and check if we have enough recipes to meet it If we don't, then we should
// identify which ones don't fit, and swap one of the protein or cuisines to
// another index and try the test again
// https://www.toptal.com/developers/algorithms/maximum-flow-linear-assignment-problem
// https://www.geeksforgeeks.org/dsa/maximum-bipartite-matching/
// https://www.geeksforgeeks.org/artificial-intelligence/constraint-satisfaction-problems-csp-in-artificial-intelligence/
// https://luluyan.medium.com/solving-real-world-puzzles-constraint-satisfaction-problems-for-data-science-applied-ai-incl-44af36cf48c5
// TODO: research above, also need to ensure that recipes are not being selected
// more than once

#include "solver.h"
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

void random_shuffle(Recipe *array, int array_size, int to_pick) {
    for (int i = 0; i < to_pick; i++) {
        int j = i + (rand() % (array_size - i));
        printf("Swapping %d and %d\n", i, j);
        Recipe temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
}

void filter_recipes(int *filtered_recipe_indexes, Recipe *recipes, Recipe required_shape,
    int num_recipes, int *filtered_recipe_length) {
    for (int i = 0; i < num_recipes; i++) {
        printf("Req Cuisine: %d, Recipe Cuisine: %d, Req Protein: %d, Recipe "
               "Protein: %d\n",
            required_shape.cuisine, recipes[i].cuisine, required_shape.protein, recipes[i].protein);
        if ((recipes[i].cuisine == required_shape.cuisine || required_shape.cuisine == 0) &&
            (recipes[i].protein == required_shape.protein || required_shape.protein == 0)) {
            printf("Filtering recipe %d\n", i);

            filtered_recipe_indexes[*filtered_recipe_length] = i;
            *filtered_recipe_length += 1;
        }
        printf("Filtered recipe indexes: %d\n", *filtered_recipe_length);
    }
    realloc(filtered_recipe_indexes, (*filtered_recipe_length - 1) * sizeof(int));
}

void select_recipes(Output *output, Recipe *recipes_to_pick, Recipe *recipes,
    int num_recipes_to_pick, int num_recipes) {
    for (int i = 0; i < num_recipes_to_pick; i++) {
        int *filtered_recipe_indexes = malloc(num_recipes * sizeof(int));
        int filtered_recipes_length = 0;
        filter_recipes(filtered_recipe_indexes, recipes, recipes_to_pick[i], num_recipes,
            &filtered_recipes_length);
        int32_t random_index = rand() % filtered_recipes_length;
        output->recipe_indices[i] = random_index;
    }
}

void insert_recipe_cuisines(Recipe *recipes_to_pick, int num_recipes_to_pick,
    CuisineRequirement *cuisine_requirements, int num_cuisine_requirements) {
    int cuisines_selected = 0;
    for (int i = 0; i < num_cuisine_requirements; i++) {
        for (int j = 0; j < cuisine_requirements[i].num_days; j++) {
            recipes_to_pick[cuisines_selected].cuisine = cuisine_requirements[i].cuisine;
            cuisines_selected++;
        }
    }
}

void insert_recipe_proteins(Recipe *recipes_to_pick, int num_recipes_to_pick,
    ProteinRequirement *protein_requirements, int num_protein_requirements) {
    int cuisines_selected = 0;
    for (int i = 0; i < num_protein_requirements; i++) {
        for (int j = 0; j < protein_requirements[i].num_days; j++) {
            recipes_to_pick[cuisines_selected].protein = protein_requirements[i].protein;
            cuisines_selected++;
        }
    }
}

bool validate_cuisine_requirements(CuisineRequirement *cuisine_requirements,
    int num_cuisine_requirements, int num_recipes_to_pick) {
    int total_cuisine_requirements = 0;
    for (int i = 0; i < num_cuisine_requirements; i++) {
        total_cuisine_requirements += cuisine_requirements[i].num_days;
    }
    if (total_cuisine_requirements > num_recipes_to_pick) {
        return false;
    }
    return true;
}

int mr_solver_ping(Input *input, Output *output) {
    Recipe *recipes_to_pick = malloc(input->num_recipes_to_pick * sizeof(Recipe));
    for (int i = 0; i < input->num_recipes_to_pick; i++) {
        recipes_to_pick[i] = (Recipe){0};
    }

    if (!validate_cuisine_requirements(input->cuisine_requirements, input->num_cuisine_requirements,
            input->num_recipes_to_pick)) {
        return 1;
    }

    insert_recipe_cuisines(recipes_to_pick, input->num_recipes_to_pick, input->cuisine_requirements,
        input->num_cuisine_requirements);
    random_shuffle(recipes_to_pick, input->num_recipes_to_pick, input->num_recipes_to_pick);
    insert_recipe_proteins(recipes_to_pick, input->num_recipes_to_pick, input->protein_requirements,
        input->num_protein_requirements);
    random_shuffle(recipes_to_pick, input->num_recipes_to_pick, input->num_recipes_to_pick);

    // printf("Recipes To Pick:\n");
    // for (int i = 0; i < input->num_recipes_to_pick; i++) {
    //     printf("Cuisine: %d Protein: %d\n", recipes_to_pick[i].cuisine,
    //     recipes_to_pick[i].protein);
    // }

    select_recipes(
        output, recipes_to_pick, input->recipes, input->num_recipes_to_pick, input->num_recipes);

    printf("Selected recipes: \n");
    for (int i = 0; i < input->num_recipes_to_pick; i++) {
        printf("Inddex: %d Cuisine: %d Protein: %d\n", output->recipe_indices[i],
            input->recipes[output->recipe_indices[i]].protein,
            input->recipes[output->recipe_indices[i]].cuisine);
    }

    return 0;
}
