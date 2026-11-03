# 3. Simple units: one unit per ingredient, no conversions

- Status: Accepted
- Date: 2026-11-03

## Context

The roadmap offers two modes for shopping-list quantities: a simple mode and an advanced mode
that understands conversions between units. The choice shapes the schema, so it had to be made
before the shopping list grew further (pending decision 6).

The code already behaves as the simple mode:

- `ingredient.unit_id` holds the one unit an ingredient is measured in. `recipe_ingredient` has
  a `qty` but deliberately no unit of its own.
- The shopping list sums `recipe_ingredient.qty` per ingredient across the plan's recipes and
  stores the result against the ingredient's current unit in `shopping_list_item`.
- A line's `qty` is nullable: a recipe may leave the amount unsaid ("salt, to taste"), and a
  list line is empty when none of its recipes gave one.

## Decision

Meal Roulette uses simple mode.

- Every ingredient has exactly one unit. Every recipe line for that ingredient is written in
  that unit.
- The shopping list sums quantities per ingredient in that unit. It never converts between
  units, and it never treats two ingredients as the same item.
- `shopping_list_item.unit_id` is a snapshot of the ingredient's unit at generation time, so a
  past list stays readable if the ingredient's unit is later changed.
- Advanced conversions are out of scope for the MVP.

## Consequences

- Summing is plain addition in SQL. There are no unit families, factors or rounding rules to
  keep correct.
- The author picks one unit per ingredient and sticks to it. A recipe that wants "2 tbsp" of an
  ingredient measured in grams has to restate the amount in grams; the app will not help.
- Changing an ingredient's unit does not rewrite existing recipe lines. Their amounts keep
  their numbers but are now read in the new unit, so changing a unit is a manual cleanup.
- The shopping list needs no migration to stay in this mode. It is also the mode that is
  cheapest to leave, because nothing stored depends on conversions existing.

## What advanced mode would need later

Moving to advanced mode is additive but not free. It would need:

- a `unit_id` on `recipe_ingredient`, so a line can be written in a unit other than the
  ingredient's default. Existing rows would be backfilled from `ingredient.unit_id`;
- a conversion model on `unit`: a dimension (mass, volume, count) and a factor to a base unit
  per dimension;
- per-ingredient density, or an explicit conversion, for crossing dimensions (cups of flour to
  grams);
- a rule for the displayed unit of a summed line, such as the base unit, the largest unit that
  keeps the amount at least 1, or the ingredient's default;
- a decision on what to do when two lines cannot be converted: keep them as separate lines or
  flag them.

The shopping-list item already stores `unit_id` beside `qty`, so the list's storage shape does
not change. Only how a line's amount is computed does.
