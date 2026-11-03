// Meal Roulette roller / constraint solver (portable C): pick recipes for the
// week under the given constraints.
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
// How a roll works: it is a constraint satisfaction problem (CSP). The days
// are the variables, each day's domain is the recipes still allowed, and every
// Requirement is a counting constraint over the picks. A backtracking search
// fills the days in order, trying each day's candidates in a seeded random
// order, and a forward check after every placement undoes a choice as soon as
// some rule can no longer be met. Required rules are searched first; then each
// preference is added, most important first, at the tightest count that still
// leaves a valid week.
//
// A SlotLimit pins what one day may take (a difficulty band or a time cap for
// that day), so those days are not interchangeable. The days with the
// fewest allowed recipes are filled first. The finished week is shuffled only
// among days that allow exactly the same recipes, which keeps every limit.
//
// Background reading:
// https://www.geeksforgeeks.org/artificial-intelligence/constraint-satisfaction-problems-csp-in-artificial-intelligence/
// https://luluyan.medium.com/solving-real-world-puzzles-constraint-satisfaction-problems-for-data-science-applied-ai-incl-44af36cf48c5

#include "solver.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// Placements one attempt may make before giving up. Proving that a rule can't
// fit is the slow case for backtracking; this keeps a roll from hanging.
#define MR_STEP_LIMIT 200000L

typedef enum { FOUND, NOT_FOUND, GAVE_UP } Outcome;

// A requirement's targets for the current attempt. Required rules are always
// on; a preference is switched on, and loosened if need be, one at a time.
typedef struct {
    bool on;
    int32_t min;
    int32_t max;
} Target;

typedef struct {
    const Input *input;
    Target *targets;
    // Picks so far that match each requirement.
    int32_t *counts;
    bool *used;
    // One list of num_recipes slots per day, so a later day never overwrites
    // the list an earlier day is still working through.
    int32_t *candidates;
    int32_t *picks;
    // eligible[day * num_recipes + recipe]: whether the day's limits allow it.
    bool *eligible;
    // The order days are filled in: fewest allowed recipes first.
    int32_t *order;
    bool has_slot_limits;
    long steps;
    bool gave_up;
    // splitmix64 state. The C library's rand() differs between platforms and
    // shares one state across threads, so a seed would not reproduce a week.
    uint64_t rng;
} Search;

static uint64_t next_random(Search *search) {
    uint64_t z = (search->rng += UINT64_C(0x9E3779B97F4A7C15));
    z = (z ^ (z >> 30)) * UINT64_C(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)) * UINT64_C(0x94D049BB133111EB);
    return z ^ (z >> 31);
}

// Uniform in [0, bound), bound > 0. Draws that land in the uneven tail of the
// 64-bit range are thrown away so no value is favoured.
static uint64_t random_below(Search *search, uint64_t bound) {
    uint64_t limit = UINT64_MAX - UINT64_MAX % bound;
    uint64_t value;
    do {
        value = next_random(search);
    } while (value >= limit);
    return value % bound;
}

static void shuffle_indices(Search *search, int32_t *array, int32_t array_size) {
    for (int32_t i = 0; i < array_size; i++) {
        int32_t j = i + (int32_t)random_below(search, (uint64_t)(array_size - i));
        int32_t temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
}

static bool matches(const Input *input, int32_t recipe, const Requirement *requirement) {
    int32_t value =
        input->attributes[(size_t)recipe * (size_t)input->num_attributes + requirement->attribute];
    return requirement->low <= value && value <= requirement->high;
}

// False when picking `recipe` would push some rule past its maximum.
static bool fits_limits(const Search *search, int32_t recipe) {
    const Input *input = search->input;
    for (int32_t j = 0; j < input->num_requirements; j++) {
        Target target = search->targets[j];
        if (target.on && target.max != MR_NO_LIMIT && search->counts[j] >= target.max &&
            matches(input, recipe, &input->requirements[j])) {
            return false;
        }
    }
    return true;
}

static bool is_eligible(const Search *search, int32_t day, int32_t recipe) {
    return search->eligible[(size_t)day * (size_t)search->input->num_recipes + (size_t)recipe];
}

// The recipes `day` may still take: allowed by its limits, unused (no recipe
// twice), and not pushing any rule past its maximum.
static void filter_recipes(const Search *search, int32_t day, int32_t *filtered_recipe_indexes,
    int32_t *filtered_recipe_length) {
    *filtered_recipe_length = 0;
    for (int32_t i = 0; i < search->input->num_recipes; i++) {
        if (!search->used[i] && is_eligible(search, day, i) && fits_limits(search, i)) {
            filtered_recipe_indexes[*filtered_recipe_length] = i;
            *filtered_recipe_length += 1;
        }
    }
}

static void adjust_counts(Search *search, int32_t recipe, int32_t change) {
    const Input *input = search->input;
    for (int32_t j = 0; j < input->num_requirements; j++) {
        if (matches(input, recipe, &input->requirements[j])) {
            search->counts[j] += change;
        }
    }
}

static void place_recipe(Search *search, int32_t day, int32_t recipe) {
    search->picks[day] = recipe;
    search->used[recipe] = true;
    adjust_counts(search, recipe, +1);
    search->steps++;
}

static void unplace_recipe(Search *search, int32_t recipe) {
    search->used[recipe] = false;
    adjust_counts(search, recipe, -1);
}

// Forward check: false once a rule's minimum can no longer fit in the days left
// or in the recipes not yet used, or a day still to fill has no unused recipe
// its limits allow, so the search can undo now instead of failing several days
// later. Maximums are enforced by filter_recipes. `next` is the position in the
// fill order of the first day still to fill.
static bool still_feasible(const Search *search, int32_t next) {
    const Input *input = search->input;
    int32_t days_left = input->num_recipes_to_pick - next;
    for (int32_t p = next; search->has_slot_limits && p < input->num_recipes_to_pick; p++) {
        bool open = false;
        for (int32_t i = 0; i < input->num_recipes && !open; i++) {
            open = !search->used[i] && is_eligible(search, search->order[p], i);
        }
        if (!open) {
            return false;
        }
    }
    for (int32_t j = 0; j < input->num_requirements; j++) {
        if (!search->targets[j].on) {
            continue;
        }
        int32_t need = search->targets[j].min - search->counts[j];
        if (need <= 0) {
            continue;
        }
        if (need > days_left) {
            return false;
        }
        int32_t available = 0;
        for (int32_t i = 0; i < input->num_recipes && available < need; i++) {
            if (!search->used[i] && matches(input, i, &input->requirements[j])) {
                available++;
            }
        }
        if (available < need) {
            return false;
        }
    }
    return true;
}

// Fills every day from fill-order position `position` to the end of the week
// and returns true once each day holds a recipe. The caller has already checked
// still_feasible() for the positions from `position` onwards.
static bool select_recipes(Search *search, int32_t position) {
    if (position == search->input->num_recipes_to_pick) {
        // The forward check after the last placement ran with 0 days left, so
        // every minimum is already met.
        return true;
    }
    if (search->steps >= MR_STEP_LIMIT) {
        search->gave_up = true;
        return false;
    }
    int32_t day = search->order[position];
    int32_t *candidates =
        search->candidates + (size_t)position * (size_t)search->input->num_recipes;
    int32_t candidates_length = 0;
    filter_recipes(search, day, candidates, &candidates_length);
    shuffle_indices(search, candidates, candidates_length);
    for (int32_t i = 0; i < candidates_length; i++) {
        place_recipe(search, day, candidates[i]);
        if (still_feasible(search, position + 1) && select_recipes(search, position + 1)) {
            return true;
        }
        unplace_recipe(search, candidates[i]);
        if (search->gave_up) {
            return false;
        }
    }
    return false;
}

// One full search under the targets as they stand. Re-seeding makes the week
// depend only on the seed and the targets, not on earlier attempts.
static Outcome attempt(Search *search) {
    const Input *input = search->input;
    search->rng = input->seed;
    memset(search->used, 0, (size_t)input->num_recipes * sizeof(bool));
    memset(search->counts, 0, (size_t)input->num_requirements * sizeof(int32_t));
    search->steps = 0;
    search->gave_up = false;
    if (still_feasible(search, 0) && select_recipes(search, 0)) {
        return FOUND;
    }
    return search->gave_up ? GAVE_UP : NOT_FOUND;
}

static void keep_week(const Search *search, int32_t *week) {
    memcpy(week, search->picks, (size_t)search->input->num_recipes_to_pick * sizeof(int32_t));
}

// Switches preference `j` on at the tightest target the recipes allow,
// loosening it one step at a time (a lower minimum, or a higher maximum), and
// switches it off again if even the loosest version fails.
static void add_preference(Search *search, int32_t j, int32_t *week) {
    const Requirement *requirement = &search->input->requirements[j];
    Target *target = &search->targets[j];
    int32_t days = search->input->num_recipes_to_pick;
    target->on = true;
    target->min = requirement->min_count;
    target->max = requirement->max_count;
    for (;;) {
        if (attempt(search) == FOUND) {
            keep_week(search, week);
            return;
        }
        if (target->min > 1) {
            target->min--;
        } else if (target->max != MR_NO_LIMIT && target->max + 1 < days) {
            target->max++;
        } else {
            break;
        }
    }
    target->on = false;
}

static bool allowed_by_limits(const Input *input, int32_t day, int32_t recipe) {
    for (int32_t j = 0; j < input->num_slot_limits; j++) {
        const SlotLimit *limit = &input->slot_limits[j];
        if (limit->day != day) {
            continue;
        }
        int32_t value = input->attributes[(size_t)recipe * (size_t)input->num_attributes +
                                          (size_t)limit->attribute];
        if (value < limit->low || value > limit->high) {
            return false;
        }
    }
    return true;
}

// Kuhn's augmenting-path step: gives `day` a recipe its limits allow, moving
// earlier days to other recipes if that is what it takes.
static bool assign_day(const Search *search, int32_t day, bool *seen, int32_t *owner) {
    for (int32_t i = 0; i < search->input->num_recipes; i++) {
        if (!is_eligible(search, day, i) || seen[i]) {
            continue;
        }
        seen[i] = true;
        if (owner[i] < 0 || assign_day(search, owner[i], seen, owner)) {
            owner[i] = day;
            return true;
        }
    }
    return false;
}

// False when no way of giving every day its own allowed recipe exists. The
// search alone would only learn that after trying a great many weeks.
static bool every_day_can_be_filled(const Search *search, bool *seen, int32_t *owner) {
    const Input *input = search->input;
    for (int32_t i = 0; i < input->num_recipes; i++) {
        owner[i] = -1;
    }
    for (int32_t day = 0; day < input->num_recipes_to_pick; day++) {
        memset(seen, 0, (size_t)input->num_recipes * sizeof(bool));
        if (!assign_day(search, day, seen, owner)) {
            return false;
        }
    }
    return true;
}

static int32_t allowed_count(const Search *search, int32_t day) {
    int32_t count = 0;
    for (int32_t i = 0; i < search->input->num_recipes; i++) {
        count += is_eligible(search, day, i) ? 1 : 0;
    }
    return count;
}

static bool same_allowed_recipes(const Search *search, int32_t a, int32_t b) {
    size_t n = (size_t)search->input->num_recipes;
    return memcmp(search->eligible + (size_t)a * n, search->eligible + (size_t)b * n,
               n * sizeof(bool)) == 0;
}

// Shuffles the picks among days that allow the same recipes, so a limit that
// holds for one of them holds for all of them.
static void shuffle_interchangeable_days(Search *search, int32_t *week) {
    int32_t days = search->input->num_recipes_to_pick;
    for (int32_t i = 0; i < days; i++) {
        int32_t same = 0;
        for (int32_t k = i; k < days; k++) {
            same += same_allowed_recipes(search, i, k) ? 1 : 0;
        }
        int32_t pick = (int32_t)random_below(search, (uint64_t)same);
        for (int32_t k = i; k < days; k++) {
            if (same_allowed_recipes(search, i, k) && pick-- == 0) {
                int32_t temp = week[i];
                week[i] = week[k];
                week[k] = temp;
                break;
            }
        }
    }
}

static bool validate_input(const Input *input, const Output *output) {
    if (input == NULL || output == NULL || output->recipe_indices == NULL) {
        return false;
    }
    if (input->num_recipes_to_pick < 1 || input->num_recipes_to_pick > input->num_recipes ||
        input->num_attributes < 0 || input->num_requirements < 0) {
        return false;
    }
    if ((input->num_attributes > 0 && input->attributes == NULL) ||
        (input->num_requirements > 0 && input->requirements == NULL)) {
        return false;
    }
    if (input->num_slot_limits < 0 || (input->num_slot_limits > 0 && input->slot_limits == NULL)) {
        return false;
    }
    for (int32_t j = 0; j < input->num_slot_limits; j++) {
        const SlotLimit *l = &input->slot_limits[j];
        if (l->day < 0 || l->day >= input->num_recipes_to_pick || l->attribute < 0 ||
            l->attribute >= input->num_attributes || l->low > l->high) {
            return false;
        }
    }
    for (int32_t j = 0; j < input->num_requirements; j++) {
        const Requirement *r = &input->requirements[j];
        if (r->attribute < 0 || r->attribute >= input->num_attributes || r->low > r->high ||
            r->min_count < 0 || r->max_count < MR_NO_LIMIT ||
            (r->max_count != MR_NO_LIMIT && r->min_count > r->max_count) || r->importance < MR_LOW ||
            r->importance > MR_REQUIRED) {
            return false;
        }
    }
    return true;
}

// malloc(0) may return NULL, which would read as running out of memory.
// calloc checks count * size for overflow, so a huge count fails instead of
// wrapping into a buffer that is too small.
static void *alloc_at_least_one(size_t count, size_t size) {
    return calloc(count > 0 ? count : 1, size);
}

int32_t mr_solver_roll(const Input *input, Output *output) {
    if (!validate_input(input, output)) {
        return MR_ERR_INVALID_INPUT;
    }

    size_t num_days = (size_t)input->num_recipes_to_pick;
    size_t num_recipes = (size_t)input->num_recipes;
    size_t num_requirements = (size_t)input->num_requirements;
    // The candidate lists hold num_days * num_recipes slots.
    if (num_recipes > SIZE_MAX / num_days) {
        return MR_ERR_NO_MEMORY;
    }
    Search search = {0};
    search.input = input;
    search.targets = alloc_at_least_one(num_requirements, sizeof(Target));
    search.counts = alloc_at_least_one(num_requirements, sizeof(int32_t));
    search.used = alloc_at_least_one(num_recipes, sizeof(bool));
    search.candidates = alloc_at_least_one(num_days * num_recipes, sizeof(int32_t));
    search.picks = alloc_at_least_one(num_days, sizeof(int32_t));
    search.eligible = alloc_at_least_one(num_days * num_recipes, sizeof(bool));
    search.order = alloc_at_least_one(num_days, sizeof(int32_t));
    search.has_slot_limits = input->num_slot_limits > 0;
    int32_t *preferences = alloc_at_least_one(num_requirements, sizeof(int32_t));
    int32_t *allowed = alloc_at_least_one(num_days, sizeof(int32_t));
    int32_t *owner = alloc_at_least_one(num_recipes, sizeof(int32_t));
    bool *seen = alloc_at_least_one(num_recipes, sizeof(bool));

    int32_t status = MR_ERR_NO_MEMORY;
    if (!search.targets || !search.counts || !search.used || !search.candidates ||
        !search.picks || !search.eligible || !search.order || !preferences || !allowed ||
        !owner || !seen) {
        goto cleanup;
    }

    for (size_t day = 0; day < num_days; day++) {
        for (size_t recipe = 0; recipe < num_recipes; recipe++) {
            search.eligible[day * num_recipes + recipe] =
                allowed_by_limits(input, (int32_t)day, (int32_t)recipe);
        }
        allowed[day] = allowed_count(&search, (int32_t)day);
        search.order[day] = (int32_t)day;
    }
    if (search.has_slot_limits) {
        // Insertion sort keeps equal days in week order, so a week with no
        // limits is filled front to back as before.
        for (int32_t i = 1; i < input->num_recipes_to_pick; i++) {
            int32_t current = search.order[i];
            int32_t k = i;
            while (k > 0 && allowed[search.order[k - 1]] > allowed[current]) {
                search.order[k] = search.order[k - 1];
                k--;
            }
            search.order[k] = current;
        }
        if (!every_day_can_be_filled(&search, seen, owner)) {
            status = MR_ERR_UNSATISFIABLE;
            goto cleanup;
        }
    }

    int32_t num_preferences = 0;
    for (int32_t j = 0; j < input->num_requirements; j++) {
        const Requirement *r = &input->requirements[j];
        bool required = r->importance == MR_REQUIRED;
        search.targets[j] = (Target){.on = required, .min = r->min_count, .max = r->max_count};
        if (!required) {
            preferences[num_preferences++] = j;
        }
    }

    Outcome outcome = attempt(&search);
    if (outcome != FOUND) {
        status = outcome == GAVE_UP ? MR_ERR_GAVE_UP : MR_ERR_UNSATISFIABLE;
        goto cleanup;
    }
    keep_week(&search, output->recipe_indices);

    // Most important first; equal levels keep the order they were given in.
    for (int32_t i = 1; i < num_preferences; i++) {
        int32_t current = preferences[i];
        int32_t k = i;
        while (k > 0 && input->requirements[preferences[k - 1]].importance <
                            input->requirements[current].importance) {
            preferences[k] = preferences[k - 1];
            k--;
        }
        preferences[k] = current;
    }
    for (int32_t i = 0; i < num_preferences; i++) {
        add_preference(&search, preferences[i], output->recipe_indices);
    }

    // The search leaves the picks a rule forces for the last days; shuffling
    // spreads them across the week.
    if (search.has_slot_limits) {
        shuffle_interchangeable_days(&search, output->recipe_indices);
    } else {
        shuffle_indices(&search, output->recipe_indices, input->num_recipes_to_pick);
    }
    status = MR_OK;

cleanup:
    free(search.targets);
    free(search.counts);
    free(search.used);
    free(search.candidates);
    free(search.picks);
    free(search.eligible);
    free(search.order);
    free(preferences);
    free(allowed);
    free(owner);
    free(seen);
    return status;
}

size_t mr_solver_input_size(void) { return sizeof(Input); }

size_t mr_solver_requirement_size(void) { return sizeof(Requirement); }

size_t mr_solver_slot_limit_size(void) { return sizeof(SlotLimit); }
